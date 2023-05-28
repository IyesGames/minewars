use crate::{assets::LocaleAssets, prelude::*};
use bevy_fluent::prelude::*;
use fluent_content::Content;
use unic_langid::LanguageIdentifier;

pub struct LocalePlugin;

impl Plugin for LocalePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            (detect_locales, init_l10n)
                .chain()
                .in_schedule(OnExit(AppState::AssetsLoading)),
        );
        app.add_system(
            resolve_l10n
                .in_set(L10nResolveSet)
                .run_if(not(in_state(AppState::AssetsLoading))),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct L10nResolveSet;

#[derive(Component)]
pub struct L10nKey(pub String);

#[derive(Resource)]
pub struct Locales(Vec<LanguageIdentifier>);

fn detect_locales(world: &mut World) {
    let locales = {
        let assets = world.resource::<LocaleAssets>();
        let bundles = world.resource::<Assets<BundleAsset>>();
        let mut locales: Vec<_> = assets
            .bundles
            .iter()
            .map(|handle| {
                let bundle = bundles.get(handle).unwrap();
                bundle.locales[0].clone()
            })
            .collect();
        locales.sort_unstable();
        locales
    };
    world.insert_resource(
        // FIXME: do actual locale selection
        Locale::new(locales[0].clone()).with_default("en-US".parse().unwrap()),
    );
    world.insert_resource(Locales(locales));
}

fn init_l10n(mut commands: Commands, l10n_builder: LocalizationBuilder, assets: Res<LocaleAssets>) {
    let l10n = l10n_builder.build(assets.bundles.iter());
    commands.insert_resource(l10n);
}

fn resolve_l10n(
    locale: Res<Locale>,
    l10n_builder: LocalizationBuilder,
    assets: Res<LocaleAssets>,
    mut ass_ev: EventReader<AssetEvent<BundleAsset>>,
    mut l10n: ResMut<Localization>,
    mut query: ParamSet<(
        Query<(&mut Text, &L10nKey), Changed<L10nKey>>,
        Query<(&mut Text, &L10nKey)>,
    )>,
) {
    let mut regenerate = false;

    if locale.is_changed() || !ass_ev.is_empty() {
        regenerate = true;
        ass_ev.clear();
    }

    if regenerate {
        *l10n = l10n_builder.build(assets.bundles.iter());
    }

    // closure for updating UI text
    let fn_update = |text: &mut Mut<Text>, key: &L10nKey| {
        if let Some(string) = l10n.content(&key.0) {
            text.sections[0].value = string;
        } else {
            text.sections[0].value = String::new();
        }
    };

    if regenerate {
        // query/update all if locale changed
        for (mut text, key) in &mut query.p1() {
            fn_update(&mut text, key);
        }
    } else {
        // only update any new/changed L10Keys otherwise
        for (mut text, key) in &mut query.p0() {
            fn_update(&mut text, key);
        }
    };
}
