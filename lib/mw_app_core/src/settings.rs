use std::any::TypeId;
use std::ffi::OsStr;

use bevy::ecs::query::{QueryData, QueryFilter, ROQueryItem};
use bevy::ecs::system::SystemParam;
use bevy::reflect::{GetTypeRegistration, TypeRegistry};
use bevy::reflect::serde::{ReflectSerializer, UntypedReflectDeserializer};
use serde::{de::DeserializeSeed, Serialize};

use crate::prelude::*;

pub static SETTINGS_ENGINE: &str = "engine.ron";
pub static SETTINGS_APP: &str = "app.ron";
pub static SETTINGS_LOCAL: & str = "local.ron";
pub static SETTINGS_USER: & str = "user.ron";

pub static SETTINGS_ALL: &[&str] = &[SETTINGS_ENGINE, SETTINGS_APP, SETTINGS_LOCAL, SETTINGS_USER];

pub mod prelude {
    pub use super::{Settings, SettingsSyncSS, SettingsAppExt};
    pub use super::{ResourceSetting, EntitySetting, GovernorSetting};
    pub use super::{SETTINGS_ENGINE, SETTINGS_APP, SETTINGS_LOCAL, SETTINGS_USER};
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SettingsStore>();
    app.configure_stage_set(
        Update,
        SettingsSyncSS,
        resource_changed::<SettingsStore>,
    );
    app.add_systems(Update,
        sync_settings.in_set(SetStage::Provide(SettingsSyncSS))
    );
}

/// StageSet for settings load/store
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SettingsSyncSS;

pub trait SettingsAppExt {
    fn init_setting<T: Setting + GetTypeRegistration + Default>(
        &mut self,
        collection: &OsStr,
    );
    fn init_setting_resource<T: ResourceSetting>(
        &mut self,
        collection: &OsStr,
    );
    fn init_setting_entity<T: EntitySetting>(
        &mut self,
        collection: &OsStr,
    );
    fn insert_setting<T: Setting + GetTypeRegistration>(
        &mut self,
        collection: &OsStr,
        value: T,
    );
}
impl SettingsAppExt for App {
    fn init_setting<T: Setting + GetTypeRegistration + Default>(
        &mut self,
        collection: &OsStr,
    ) {
        self.init_resource::<SettingsStore>();
        self.register_type::<T>();
        self.world.resource_mut::<SettingsStore>()
            .init_setting::<T>(collection);
    }
    fn init_setting_resource<T: ResourceSetting>(
        &mut self,
        collection: &OsStr,
    ) {
        self.init_setting::<ResourceSettingS<T>>(collection)
    }
    fn init_setting_entity<T: EntitySetting>(
        &mut self,
        collection: &OsStr,
    ) {
        self.init_setting::<EntitySettingS<T>>(collection)
    }
    fn insert_setting<T: Setting + GetTypeRegistration>(
        &mut self,
        collection: &OsStr,
        value: T,
    ) {
        self.init_resource::<SettingsStore>();
        self.register_type::<T>();
        self.world.resource_mut::<SettingsStore>()
            .insert_setting(collection, value);
    }
}

/// Marker component for the settings governor entity
#[derive(Component)]
pub struct SettingsGovernor;

#[derive(SystemParam)]
pub struct Settings<'w, 's, T: QueryData + 'static, F: QueryFilter + 'static = ()> {
    query_settings: Query<'w, 's, T, (F, With<SettingsGovernor>)>,
}

impl<'w, 's, T: QueryData + 'static, F: QueryFilter + 'static> Settings<'w, 's, T, F> {
    pub fn get(&self) -> Option<ROQueryItem<'_, T>> {
        self.query_settings.get_single().ok()
    }
    pub fn get_mut(&mut self) -> Option<T::Item<'_>> {
        self.query_settings.get_single_mut().ok()
    }
    pub fn is_available(&self) -> bool {
        !self.query_settings.is_empty()
    }
}

#[derive(Resource, Default)]
pub struct SettingsStore {
    settings_root: Option<PathBuf>,
    collections: HashMap<Arc<OsStr>, HashSet<TypeId>>,
    map: HashMap<TypeId, SettingsEntry>,
}

struct SettingsEntry {
    data: Box<dyn Setting>,
    collection: Arc<OsStr>,
}

impl SettingsStore {
    pub fn init_setting<T: Setting + Default>(
        &mut self,
        collection: &OsStr,
    ) {
        if self.map.contains_key(&TypeId::of::<T>()) {
            return;
        }
        self.insert_setting(collection, T::default());
    }
    pub fn init_setting_resource<T: ResourceSetting>(
        &mut self,
        collection: &OsStr,
    ) {
        self.init_setting::<ResourceSettingS<T>>(collection)
    }
    pub fn init_setting_entity<T: EntitySetting>(
        &mut self,
        collection: &OsStr,
    ) {
        self.init_setting::<EntitySettingS<T>>(collection)
    }
    pub fn insert_setting<T: Setting>(
        &mut self,
        collection: &OsStr,
        value: T,
    ) {
        let arc = if let Some((a, ids)) = self.collections.get_key_value_mut(collection) {
            ids.insert(TypeId::of::<T>());
            a.clone()
        } else {
            let a: Arc<OsStr> = collection.into();
            let mut ids = HashSet::new();
            ids.insert(TypeId::of::<T>());
            self.collections.insert(a.clone(), ids);
            a
        };
        self.map.insert(
            TypeId::of::<T>(),
            SettingsEntry {
                data: Box::new(value),
                collection: arc,
            }
        );
    }
    pub fn insert_setting_dyn(
        &mut self,
        collection: &OsStr,
        type_id: TypeId,
        value: Box<dyn Setting>,
    ) {
        let arc = if let Some((a, ids)) = self.collections.get_key_value_mut(collection) {
            ids.insert(type_id);
            a.clone()
        } else {
            let a: Arc<OsStr> = collection.into();
            let mut ids = HashSet::new();
            ids.insert(type_id);
            self.collections.insert(a.clone(), ids);
            a
        };
        self.map.insert(
            type_id,
            SettingsEntry {
                data: value,
                collection: arc,
            }
        );
    }
    pub fn iter_settings(&self) -> impl Iterator<Item = (&dyn Setting, &Arc<OsStr>)> {
        self.map.values().map(|v| (&*v.data, &v.collection))
    }
    pub fn iter_settings_mut(&mut self) -> impl Iterator<Item = (&mut dyn Setting, &Arc<OsStr>)> {
        self.map.values_mut().map(|v| (&mut *v.data, &v.collection))
    }
    fn load_collection(&mut self, registry: &TypeRegistry, collection: &OsStr) -> AnyResult<()> {
        let Some(dir) = &self.settings_root else {
            bail!("Don't know where config files should be saved!");
        };
        let path = dir.join(collection);
        let bytes = std::fs::read(&path)
            .with_context(|| format!("Could not read settings file: {:?}", path))?;
        let s = std::str::from_utf8(&bytes)
            .with_context(|| format!("Settings file {:?} is not UTF-8", path))?;
        let ron: ron::Value = ron::from_str(s)
            .with_context(|| format!("Settings file {:?} is not in RON format", path))?;
        let ron::Value::Seq(entries) = ron else {
            bail!("Settings file {:?} does not contain a RON sequence", path);
        };

        for entry in entries {
            // Convert back to a string and re-deserialize. Is there an easier way?
            let entry = ron::to_string(&entry).unwrap();
            let mut deserializer = ron::Deserializer::from_str(&entry).unwrap();

            let reflect_deserializer = UntypedReflectDeserializer::new(&registry);
            let output: Box<dyn Reflect> =
                reflect_deserializer.deserialize(&mut deserializer)
                    .context("Settings entry is not in valid Bevy Reflect format")?;
            let type_info = output.get_represented_type_info()
                .context("Settings entry bad type info")?;
            let type_path = type_info.type_path();
            let type_id = type_info.type_id();
            let reflect_from_reflect = registry
                .get_type_data::<bevy::reflect::ReflectFromReflect>(type_id)
                .context("type id has no RFR type data")?;
            let reflect_setting = registry
                .get_type_data::<ReflectSetting>(type_id)
                .context("type id has no Setting type data")?;
            let value: Box<dyn Reflect> = reflect_from_reflect.from_reflect(&*output)
                .context("RFR fail")?;
            let setting: Box<dyn Setting> = reflect_setting.get_boxed(value).ok()
                .with_context(|| format!("{:?} is not valid Setting", type_path))?;
            self.insert_setting_dyn(collection, type_id, setting);
        }
        Ok(())
    }
    fn store_collection(&self, registry: &TypeRegistry, collection: &OsStr) -> AnyResult<()> {
        let Some(dir) = &self.settings_root else {
            bail!("Don't know where config files should be saved!");
        };
        let path = dir.join(collection);

        let mut output = vec![b'['];
        let iter_reflect;
        if let Some(coll) = self.collections.get(collection) {
            iter_reflect = coll.iter()
                .filter_map(|tid| self.map.get(tid))
                .map(|s| &*s.data);
        } else {
            bail!("Unknown settings collection {:?}", collection);
        }
        for value in iter_reflect {
            let serializer = ReflectSerializer::new(value, &registry);
            let mut ser = ron::ser::Serializer::with_options(&mut output, Some(default()), default())
                .context("Cannot set up RON serialization")?;
            serializer.serialize(&mut ser)
                .context("Cannot serialize setting to RON")?;
            output.push(b',');
        }
        output.push(b']');

        std::fs::write(&path, &output)
            .with_context(|| format!("Cannot write to settings file {:?}", path))?;

        Ok(())
    }
    fn init_settings_root(&mut self) {
        if self.settings_root.is_some() {
            return;
        }
        self.settings_root = directories::ProjectDirs::from(
            "com", "IyesGames", "MineWars",
        ).map(|dirs| dirs.preference_dir().to_owned());
    }
}

#[reflect_trait]
pub trait Setting: Reflect {
    fn extract(&mut self, world: &mut World) -> bool;
    fn apply(&self, world: &mut World);
    fn apply_app(&self, app: &mut App) {
        self.apply(&mut app.world);
    }
}

pub trait ResourceSetting: Resource + Default + Clone + Reflect + FromReflect + TypePath {
    fn apply(&self, world: &mut World) {
    }
}

#[derive(Clone, Default, Reflect)]
pub struct ResourceSettingS<T: ResourceSetting>(T);

impl<T: ResourceSetting> Setting for ResourceSettingS<T> {
    fn extract(&mut self, world: &mut World) -> bool {
        if !world.is_resource_changed::<T>() {
            return false;
        }
        self.0 = world.resource::<T>().clone();
        self.0.apply(world);
        true
    }
    fn apply(&self, world: &mut World) {
        if let Some(mut r) = world.get_resource_mut::<T>() {
            *r = self.0.clone();
        } else {
            world.insert_resource(self.0.clone());
        }
        self.0.apply(world);
    }
}

pub trait EntitySetting: Component + Default + Clone + Reflect + FromReflect + TypePath {
    type Filter: QueryFilter;
    fn apply(&self, world: &mut World) {
    }
}

#[derive(Clone, Default, Reflect)]
pub struct EntitySettingS<T: EntitySetting>(T);

impl<T: EntitySetting> Setting for EntitySettingS<T> {
    fn extract(&mut self, world: &mut World) -> bool {
        let mut q = world.query_filtered::<Ref<T>, T::Filter>();
        if let Ok(v) = q.get_single(world) {
            if v.is_changed() {
                self.0 = v.clone();
                self.0.apply(world);
                return true;
            }
        }
        false
    }
    fn apply(&self, world: &mut World) {
        let mut q = world.query_filtered::<&mut T, T::Filter>();
        if let Ok(mut v) = q.get_single_mut(world) {
            *v = self.0.clone();
        }
        self.0.apply(world);
    }
}

pub trait GovernorSetting: Component + Default + Clone + Reflect + FromReflect + TypePath {
    fn apply(&self, world: &mut World) {
    }
}

impl<T: GovernorSetting> EntitySetting for T {
    type Filter = With<SettingsGovernor>;
    fn apply(&self, world: &mut World) {
        <Self as GovernorSetting>::apply(&self, world);
    }
}

fn sync_settings(world: &mut World, mut ranonce: Local<bool>) {
    let Some(mut settings_store) = world.remove_resource::<SettingsStore>() else {
        panic!("SettingsStore should be initialized");
    };
    settings_store.init_settings_root();
    let mut changed_collections: HashSet<Arc<OsStr>> = Default::default();
    for (setting, collection) in settings_store.iter_settings_mut() {
        if setting.extract(world) || !*ranonce {
            changed_collections.insert(collection.clone());
        }
    }
    let registry = world.resource::<AppTypeRegistry>();
    let registry = registry.read();
    for collection in changed_collections.drain() {
        info!("Settings collection {:?} has changed entries! Storing to disk...", collection);
        if let Err(e) = settings_store.store_collection(&registry, collection.as_ref()) {
            error!("Failed to store settings collection {:?}: {:#}", collection, e);
        }
    }
    std::mem::drop(registry);
    world.insert_resource(settings_store);
    *ranonce = true;
}

pub fn early_load_settings(app: &mut App, filenames: &[&str]) {
    let Some(mut settings_store) = app.world.remove_resource::<SettingsStore>() else {
        panic!("SettingsStore should be initialized");
    };
    settings_store.init_settings_root();
    let registry = app.world.resource::<AppTypeRegistry>();
    let registry = registry.read();
    for collection in filenames {
        if let Err(e) = settings_store.load_collection(&registry, collection.as_ref()) {
            error!("Error early-loading settings collection {:?}: {:#}", collection, e);
        }
    }
    std::mem::drop(registry);
    for (setting, _) in settings_store.iter_settings() {
        setting.apply_app(app);
    }
    app.world.insert_resource(settings_store);
}
