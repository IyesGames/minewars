use std::any::TypeId;
use std::ffi::OsStr;

use bevy::ecs::component::Tick;
use bevy::ecs::system::{SystemChangeTick, SystemParam};
use bevy::reflect::{GetTypeRegistration, TypeRegistry};
use bevy::reflect::serde::{ReflectSerializer, UntypedReflectDeserializer};
use serde::{de::DeserializeSeed, Serialize};

use crate::prelude::*;

/// Settings to load very early, needed to set up App, Bevy, etc.
pub static SETTINGS_ENGINE: &str = "engine.ron";
/// Settings for internal app use.
pub static SETTINGS_APP: &str = "app.ron";
/// User-facing settings specific to the local install.
pub static SETTINGS_LOCAL: & str = "local.ron";
/// User-facing settings that might be worth syncing between game installs.
pub static SETTINGS_USER: & str = "user.ron";

pub static SETTINGS_ALL: &[&str] = &[SETTINGS_ENGINE, SETTINGS_APP, SETTINGS_LOCAL, SETTINGS_USER];

pub mod prelude {
    pub use super::{Setting, ReflectSetting, Settings, SettingsMut, SettingsSyncSS, SettingsAppExt};
    pub use super::{SETTINGS_ENGINE, SETTINGS_APP, SETTINGS_LOCAL, SETTINGS_USER};
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SettingsStore>();
    app.configure_stage_set(
        Update,
        SettingsSyncSS,
        resource_changed::<SettingsStore>,
    );
    app.configure_stage_set_no_rc(
        Startup,
        SettingsSyncSS,
    );
    app.add_systems(Update,
        sync_settings
            .in_set(SetStage::Provide(SettingsSyncSS))
    );
    app.add_systems(Startup,
        apply_initial_settings
            .in_set(SetStage::Provide(SettingsSyncSS))
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
            .init_setting::<T>(collection, Tick::new(0));
    }
    fn insert_setting<T: Setting + GetTypeRegistration>(
        &mut self,
        collection: &OsStr,
        value: T,
    ) {
        self.init_resource::<SettingsStore>();
        self.register_type::<T>();
        self.world.resource_mut::<SettingsStore>()
            .insert_setting(collection, value, Tick::new(0));
    }
}

#[derive(Resource, Default)]
pub struct SettingsStore {
    settings_root: Option<PathBuf>,
    collections: HashMap<Arc<OsStr>, (HashSet<TypeId>, Tick)>,
    map: HashMap<TypeId, SettingsEntry>,
}

struct SettingsEntry {
    data: Box<dyn Setting>,
    collection: Arc<OsStr>,
    change_tick: Tick,
}

impl SettingsStore {
    pub fn init_setting<T: Setting + Default>(
        &mut self,
        collection: &OsStr,
        change_tick: Tick,
    ) {
        if self.map.contains_key(&TypeId::of::<T>()) {
            return;
        }
        self.insert_setting(collection, T::default(), change_tick);
    }
    pub fn insert_setting<T: Setting>(
        &mut self,
        collection: &OsStr,
        value: T,
        change_tick: Tick,
    ) {
        let arc = if let Some((a, (ids, tick))) = self.collections.get_key_value_mut(collection) {
            ids.insert(TypeId::of::<T>());
            *tick = change_tick;
            a.clone()
        } else {
            let a: Arc<OsStr> = collection.into();
            let mut ids = HashSet::new();
            ids.insert(TypeId::of::<T>());
            self.collections.insert(a.clone(), (ids, change_tick));
            a
        };
        self.map.insert(
            TypeId::of::<T>(),
            SettingsEntry {
                data: Box::new(value),
                collection: arc,
                change_tick,
            }
        );
    }
    pub fn insert_setting_dyn(
        &mut self,
        collection: &OsStr,
        type_id: TypeId,
        value: Box<dyn Setting>,
        change_tick: Tick,
    ) {
        let arc = if let Some((a, (ids, tick))) = self.collections.get_key_value_mut(collection) {
            ids.insert(type_id);
            *tick = change_tick;
            a.clone()
        } else {
            let a: Arc<OsStr> = collection.into();
            let mut ids = HashSet::new();
            ids.insert(type_id);
            self.collections.insert(a.clone(), (ids, change_tick));
            a
        };
        self.map.insert(
            type_id,
            SettingsEntry {
                data: value,
                collection: arc,
                change_tick,
            }
        );
    }
    pub fn iter_settings(&self) -> impl Iterator<Item = (&dyn Setting, &Arc<OsStr>)> {
        self.map.values().map(|v| (&*v.data, &v.collection))
    }
    pub fn iter_settings_mut(&mut self) -> impl Iterator<Item = (&mut dyn Setting, &Arc<OsStr>)> {
        self.map.values_mut().map(|v| (&mut *v.data, &v.collection))
    }
    pub fn get<T: Setting>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|val| val.data.as_reflect().downcast_ref())
    }
    pub fn get_mut<T: Setting>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|val| val.data.as_reflect_mut().downcast_mut())
    }
    pub fn change_tick<T: Setting>(&self) -> Option<Tick> {
        self.map
            .get(&TypeId::of::<T>())
            .map(|val| val.change_tick)
    }
    fn load_collection(&mut self, registry: &TypeRegistry, collection: &OsStr, change_tick: Tick) -> AnyResult<()> {
        let Some(dir) = &self.settings_root else {
            bail!("Don't know where config files should be saved!");
        };
        let path = dir.join(collection);
        let bytes = std::fs::read(&path)
            .with_context(|| format!("Could not read settings file: {:?}", path))?;
        let s = std::str::from_utf8(&bytes)
            .with_context(|| format!("Settings file {:?} is not UTF-8", path))?;

        let mut deserializer = ron::Deserializer::from_str(&s).unwrap();
        while deserializer.end().is_err() {
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
            self.insert_setting_dyn(collection, type_id, setting, change_tick);
        }
        Ok(())
    }
    fn store_collection(&self, registry: &TypeRegistry, collection: &OsStr) -> AnyResult<()> {
        let Some(dir) = &self.settings_root else {
            bail!("Don't know where config files should be saved!");
        };
        let path = dir.join(collection);

        let mut output = vec![];
        let iter_reflect;
        if let Some((coll, _)) = self.collections.get(collection) {
            iter_reflect = coll.iter()
                .filter_map(|tid| self.map.get(tid))
                .map(|s| &*s.data);
        } else {
            bail!("Unknown settings collection {:?}", collection);
        }
        for value in iter_reflect {
            let mut ser = ron::ser::Serializer::with_options(&mut output, Some(default()), default())
                .context("Cannot set up RON serialization")?;
            let serializer = ReflectSerializer::new(value, &registry);
            serializer.serialize(&mut ser)
                .context("Cannot serialize setting to RON")?;
            if cfg!(target_os = "windows") {
                output.push(b'\r');
                output.push(b'\n');
            } else {
                output.push(b'\n');
            }
        }

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

#[derive(SystemParam)]
pub struct Settings<'w> {
    store: Res<'w, SettingsStore>,
    change_tick: SystemChangeTick,
}

#[derive(SystemParam)]
pub struct SettingsMut<'w> {
    store: ResMut<'w, SettingsStore>,
    change_tick: SystemChangeTick,
}

impl<'w> Settings<'w> {
    pub fn get<T: Setting>(&self) -> Option<&T> {
        self.store.get::<T>()
    }
    pub fn change_tick<T: Setting>(&self) -> Option<Tick> {
        self.store.change_tick::<T>()
    }
    pub fn is_changed<T: Setting>(&self) -> bool {
        let tick = self.change_tick::<T>().unwrap_or(Tick::new(0));
        tick.is_newer_than(self.change_tick.last_run(), self.change_tick.this_run())
    }
    pub fn iter_settings(&self) -> impl Iterator<Item = (&dyn Setting, &Arc<OsStr>)> {
        self.store.iter_settings()
    }
}

impl<'w> SettingsMut<'w> {
    pub fn get<T: Setting>(&self) -> Option<&T> {
        self.store.get::<T>()
    }
    pub fn get_mut<T: Setting>(&mut self) -> Option<&mut T> {
        self.store.get_mut::<T>()
    }
    pub fn change_tick<T: Setting>(&self) -> Option<Tick> {
        self.store.change_tick::<T>()
    }
    pub fn is_changed<T: Setting>(&self) -> bool {
        let tick = self.change_tick::<T>().unwrap_or(Tick::new(0));
        tick.is_newer_than(self.change_tick.last_run(), self.change_tick.this_run())
    }
    pub fn iter_settings(&self) -> impl Iterator<Item = (&dyn Setting, &Arc<OsStr>)> {
        self.store.iter_settings()
    }
    pub fn iter_settings_mut(&mut self) -> impl Iterator<Item = (&mut dyn Setting, &Arc<OsStr>)> {
        self.store.iter_settings_mut()
    }
    pub fn init_setting<T: Setting + Default>(
        &mut self,
        collection: &OsStr,
    ) {
        self.store.init_setting::<T>(collection, self.change_tick.this_run())
    }
    pub fn insert_setting<T: Setting>(
        &mut self,
        collection: &OsStr,
        value: T,
    ) {
        self.store.insert_setting(collection, value, self.change_tick.this_run())
    }
    pub fn insert_setting_dyn(
        &mut self,
        collection: &OsStr,
        type_id: TypeId,
        value: Box<dyn Setting>,
        change_tick: Tick,
    ) {
        self.store.insert_setting_dyn(collection, type_id, value, self.change_tick.this_run())
    }
}

#[reflect_trait]
pub trait Setting: Reflect {
    fn apply(&self, world: &mut World) {
    }
}

fn sync_settings(world: &mut World, mut ranonce: Local<bool>) {
    let Some(mut settings_store) = world.remove_resource::<SettingsStore>() else {
        panic!("SettingsStore should be initialized");
    };
    settings_store.init_settings_root();
    let mut changed_collections: HashSet<Arc<OsStr>> = Default::default();
    let tick_this_run = world.change_tick();
    let tick_last_run = world.last_change_tick();
    for (coll, (_, tick)) in settings_store.collections.iter() {
        if tick.is_newer_than(tick_last_run, tick_this_run) {
            changed_collections.insert(coll.clone());
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
        if let Err(e) = settings_store.load_collection(&registry, collection.as_ref(), Tick::new(0)) {
            eprintln!("Error early-loading settings collection {:?}: {:#}", collection, e);
        }
    }
    std::mem::drop(registry);
    app.world.insert_resource(settings_store);
}

fn apply_initial_settings(world: &mut World) {
    let Some(settings_store) = world.remove_resource::<SettingsStore>() else {
        panic!("SettingsStore should be initialized");
    };
    for (setting, _) in settings_store.iter_settings() {
        setting.apply(world);
    }
    world.insert_resource(settings_store);
}

pub fn apply_all_changed_settings(world: &mut World) {
    let Some(settings_store) = world.remove_resource::<SettingsStore>() else {
        panic!("SettingsStore should be initialized");
    };
    let tick_this_run = world.change_tick();
    let tick_last_run = world.last_change_tick();
    for setting in settings_store.map.values() {
        if setting.change_tick.is_newer_than(tick_last_run, tick_this_run) {
            setting.data.apply(world);
        }
    }
    world.insert_resource(settings_store);
}

pub fn apply_setting<T: Setting>(world: &mut World) {
    let Some(settings_store) = world.remove_resource::<SettingsStore>() else {
        panic!("SettingsStore should be initialized");
    };
    if let Some(setting) = settings_store.map.get(&TypeId::of::<T>()) {
        setting.data.apply(world);
    }
    world.insert_resource(settings_store);
}
