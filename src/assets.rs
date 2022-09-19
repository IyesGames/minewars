use crate::prelude::*;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::reflect::TypeUuid;
use bevy_common_assets::toml::TomlAssetPlugin;

use crate::AppGlobalState;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(
            ProgressPlugin::new(AppGlobalState::AssetsLoading)
                // .continue_to(AppGlobalState::SplashIyes)
        );
        app.add_loading_state(
            LoadingState::new(AppGlobalState::AssetsLoading)
                .continue_to_state(AppGlobalState::SplashIyes)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "ui.assets",
                    "logos.assets",
                    "game.assets",
                ])
                .with_collection::<UiAssets>()
                .with_collection::<Splashes>()
                .with_collection::<TitleLogo>()
                .with_collection::<TileAssets>()
        );
        app.add_plugin(TomlAssetPlugin::<ZoomLevelsAsset>::new(&["zoomlevels.toml"]));
        app.add_system_to_stage(CoreStage::Last, debug_progress.run_in_state(AppGlobalState::AssetsLoading));
        app.add_enter_system(AppGlobalState::AssetsLoading, setup_loadscreen);
        app.add_exit_system(AppGlobalState::AssetsLoading, despawn_with_recursive::<LoadscreenCleanup>);
        app.add_enter_system(AppGlobalState::SplashIyes, splash_init_iyes);
        app.add_exit_system(AppGlobalState::SplashIyes, despawn_with_recursive::<SplashCleanup>);
        app.add_exit_system(AppGlobalState::SplashIyes, remove_resource::<SplashNext>);
        app.add_enter_system(AppGlobalState::SplashBevy, splash_init_bevy);
        app.add_exit_system(AppGlobalState::SplashBevy, despawn_with_recursive::<SplashCleanup>);
        app.add_exit_system(AppGlobalState::SplashBevy, remove_resource::<SplashNext>);
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(AppGlobalState::SplashIyes)
                .with_system(splash_skip)
                .with_system(splash_fade)
                .into()
        );
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(AppGlobalState::SplashBevy)
                .with_system(splash_skip)
                .with_system(splash_fade)
                .into()
        );
        app.add_exit_system(AppGlobalState::SplashBevy, remove_resource::<Splashes>);
        app.add_system_to_stage(CoreStage::PostUpdate, update_loading_pct.run_in_state(AppGlobalState::AssetsLoading));
    }
}

fn debug_progress(
    counter: Res<ProgressCounter>,
) {
    let progress = counter.progress();
    trace!("Progress: {}/{}", progress.done, progress.total);
    let progress = counter.progress_complete();
    trace!("Full Progress: {}/{}", progress.done, progress.total);
}

#[derive(AssetCollection)]
pub struct UiAssets {
    #[asset(key = "font.regular")]
    pub font_regular: Handle<Font>,
    #[asset(key = "font.bold")]
    pub font_bold: Handle<Font>,
    #[asset(key = "font.light")]
    pub font_light: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct Splashes {
    #[asset(key = "logo.iyes.head")]
    pub logo_iyeshead: Handle<Image>,
    #[asset(key = "logo.iyes.text")]
    pub logo_iyestext: Handle<Image>,
    #[asset(key = "logo.bevy")]
    pub logo_bevy: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct TitleLogo {
    #[asset(key = "logo.title")]
    pub image: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct TileAssets {
    #[asset(key = "zoomlevels")]
    pub zoomlevels_handle: Handle<ZoomLevelsAsset>,
    pub zoomlevels: ZoomLevelsAsset,
    #[asset(key = "sprites.tiles6", collection(typed))]
    pub tiles6: Vec<Handle<Image>>,
    #[asset(key = "sprites.tiles4", collection(typed))]
    pub tiles4: Vec<Handle<Image>>,
    #[asset(key = "sprites.roads6", collection(typed))]
    pub roads6: Vec<Handle<Image>>,
    #[asset(key = "sprites.roads4", collection(typed))]
    pub roads4: Vec<Handle<Image>>,
    #[asset(key = "sprites.digits", collection(typed))]
    pub digits: Vec<Handle<Image>>,
    #[asset(key = "sprites.gents", collection(typed))]
    pub gents: Vec<Handle<Image>>,
    #[asset(key = "sprites.flags", collection(typed))]
    pub flags: Vec<Handle<Image>>,
}

#[derive(Debug, Clone)]
#[derive(serde::Deserialize)]
pub struct ZoomLevelDescriptor {
    pub size: u32,
    pub offset4: (f32, f32),
    pub offset6: (f32, f32),
}

#[derive(Debug, Clone, Deref, serde::Deserialize, TypeUuid)]
#[uuid = "09b0abbf-c551-49d1-8ea5-d6722eeac41f"]
pub struct ZoomLevelsAsset {
    pub zoom: Vec<ZoomLevelDescriptor>,
}

impl FromWorld for ZoomLevelsAsset {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<Assets<ZoomLevelsAsset>>();
        assets.iter().nth(0).unwrap().1.clone()
    }
}

#[derive(Component)]
struct LoadscreenCleanup;
#[derive(Component)]
struct LoadingProgressIndicator;

fn setup_loadscreen(
    mut commands: Commands,
) {
    commands.spawn_bundle(Camera2dBundle::default())
        .insert(LoadscreenCleanup);

    let container = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::GRAY),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Percent(48.0),
                top: Val::Percent(48.0),
                left: Val::Percent(20.0),
                right: Val::Percent(20.0),
            },
            padding: UiRect::all(Val::Px(2.0)),
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).insert(LoadscreenCleanup).id();

    let inner = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::WHITE),
        style: Style {
            size: Size::new(Val::Percent(0.0), Val::Percent(100.0)),
            ..Default::default()
        },
        ..Default::default()
    }).insert(LoadingProgressIndicator).id();

    commands.entity(container).push_children(&[inner]);
}

fn update_loading_pct(
    mut q: Query<&mut Style, With<LoadingProgressIndicator>>,
    progress: Res<ProgressCounter>,
) {
    let progress: f32 = progress.progress().into();
    for mut style in q.iter_mut() {
        style.size.width = Val::Percent(progress * 100.0);
    }
}

#[derive(Component)]
struct SplashCleanup;

struct SplashNext(AppGlobalState);

fn splash_init_iyes(
    mut commands: Commands,
    splashes: Res<Splashes>,
) {
    commands.insert_resource(SplashNext(AppGlobalState::SplashBevy));
    commands.spawn_bundle(Camera2dBundle::default())
        .insert(SplashCleanup);
    commands.spawn_bundle(SpriteBundle {
        texture: splashes.logo_iyeshead.clone(),
        transform: Transform::from_xyz(0.0, 75.0, 0.0),
        ..Default::default()
    }).insert(SplashCleanup)
    .insert(SplashFade::new(0.0, 0.0, 1.25, 1.5));
    commands.spawn_bundle(SpriteBundle {
        texture: splashes.logo_iyestext.clone(),
        transform: Transform::from_xyz(0.0, -175.0, 0.0),
        ..Default::default()
    }).insert(SplashCleanup)
    .insert(SplashFade::new(0.25, 0.75, 0.25, 1.75));
}

fn splash_init_bevy(
    mut commands: Commands,
    splashes: Res<Splashes>,
) {
    commands.insert_resource(SplashNext(AppGlobalState::MainMenu));
    commands.spawn_bundle(Camera2dBundle::default())
        .insert(SplashCleanup);
    commands.spawn_bundle(SpriteBundle {
        texture: splashes.logo_bevy.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    }).insert(SplashCleanup)
    .insert(SplashFade::new(0.0, 0.5, 1.0, 1.5));
}

#[derive(Component)]
struct SplashFade {
    timer_wait: Timer,
    timer_intro: Timer,
    timer_on: Timer,
    timer_fade: Timer,
}

impl SplashFade {
    fn new(wait: f32, intro: f32, on: f32, fade: f32) -> Self {
        Self {
            timer_wait: Timer::from_seconds(wait, false),
            timer_intro: Timer::from_seconds(intro, false),
            timer_on: Timer::from_seconds(on, false),
            timer_fade: Timer::from_seconds(fade, false),
        }
    }
}

fn splash_fade(
    mut q: Query<(&mut Sprite, &mut SplashFade)>,
    mut commands: Commands,
    t: Res<Time>,
    next: Res<SplashNext>,
) {
    let mut all_finished = true;
    let mut count = 0;
    for (mut sprite, mut fade) in q.iter_mut() {
        count += 1;
        if fade.timer_wait.duration().as_secs_f32() > 0.0 && !fade.timer_wait.finished() {
            fade.timer_wait.tick(t.delta());
            all_finished = false;
            sprite.color.set_a(0.0);
        } else if fade.timer_intro.duration().as_secs_f32() > 0.0 && !fade.timer_intro.finished() {
            fade.timer_intro.tick(t.delta());
            all_finished = false;
            let remain = fade.timer_intro.percent();
            sprite.color.set_a(remain);
        } else if !fade.timer_on.finished() {
            fade.timer_on.tick(t.delta());
            all_finished = false;
            sprite.color.set_a(1.0);
        } else if !fade.timer_fade.finished() {
            fade.timer_fade.tick(t.delta());
            all_finished = false;
            let remain = fade.timer_fade.percent_left();
            sprite.color.set_a(remain);
        }
    }
    if all_finished && count > 0 {
        commands.insert_resource(NextState(next.0));
    }
}

fn splash_skip(
    mut commands: Commands,
    mut kbd: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut gamepad: EventReader<GamepadEvent>,
    mut touch: EventReader<TouchInput>,
) {
    use bevy::input::ButtonState;
    use bevy::input::touch::TouchPhase;

    let mut done = false;

    for ev in kbd.iter() {
        if let ButtonState::Pressed = ev.state {
            done = true;
        }
    }

    for ev in mouse.iter() {
        if let ButtonState::Pressed = ev.state {
            done = true;
        }
    }

    for ev in gamepad.iter() {
        if let GamepadEventType::ButtonChanged(_, _) = ev.event_type {
            done = true;
        }
    }

    for ev in touch.iter() {
        if let TouchPhase::Started = ev.phase {
            done = true;
        }
    }

    if done {
        commands.insert_resource(NextState(AppGlobalState::MainMenu));
    }
}
