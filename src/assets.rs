use crate::prelude::*;
use bevy::math::const_vec2;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;

use crate::AppGlobalState;

pub const TILESZ: Vec2 = const_vec2!([256.0, 256.0]);

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_assets);
        app.add_plugin(
            ProgressPlugin::new(AppGlobalState::AssetsLoading)
                .continue_to(AppGlobalState::SplashIyes)
                .track_assets()
        );
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

pub struct UiAssets {
    pub font_regular: Handle<Font>,
    pub font_bold: Handle<Font>,
    pub font_light: Handle<Font>,
}

pub struct Splashes {
    pub logo_iyeshead: Handle<Image>,
    pub logo_iyestext: Handle<Image>,
    pub logo_bevy: Handle<Image>,
}

pub struct TitleLogo {
    pub image: Handle<Image>,
}

pub struct TileAssets {
    pub tiles: Handle<Image>,
    pub atlas: Handle<TextureAtlas>,
}

fn load_assets(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut ast: ResMut<AssetsLoading>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    // UI FONT
    let font_regular = ass.load("Sansation-Regular.ttf");
    ast.add(&font_regular);
    let font_bold = ass.load("Sansation-Bold.ttf");
    ast.add(&font_bold);
    let font_light = ass.load("Sansation-Light.ttf");
    ast.add(&font_light);

    commands.insert_resource(UiAssets {
        font_regular,
        font_bold,
        font_light,
    });

    // SPLASH LOGOS

    let logo_iyeshead = ass.load("logo_iyeshead.png");
    ast.add(&logo_iyeshead);
    let logo_iyestext = ass.load("logo_iyestext.png");
    ast.add(&logo_iyeshead);
    let logo_bevy = ass.load("logo_bevy.png");
    ast.add(&logo_bevy);

    commands.insert_resource(Splashes {
        logo_bevy,
        logo_iyeshead,
        logo_iyestext,
    });

    // TITLE
    let logo_title = ass.load("logo_minewars.png");
    ast.add(&logo_title);

    commands.insert_resource(TitleLogo {
        image: logo_title,
    });

    // TILESET

    let tiles: Handle<Image> = ass.load("tiles.ktx2");
    ast.add(&tiles);

    let atlas = TextureAtlas::from_grid(
        tiles.clone(),
        TILESZ,
        8, 8,
    );

    let atlas = atlases.add(atlas);

    commands.insert_resource(TileAssets {
        tiles,
        atlas,
    });
}

#[derive(Component)]
struct LoadscreenCleanup;
#[derive(Component)]
struct LoadingPctText;

fn setup_loadscreen(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
) {
    let top = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(0.0),
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
            },
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).insert(LoadscreenCleanup).id();

    let inner = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            margin: Rect::all(Val::Auto),
            padding: Rect::all(Val::Px(4.0)),
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let inner_pct = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            margin: Rect::all(Val::Auto),
            padding: Rect::all(Val::Px(4.0)),
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let txt_loading = commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            "Loading???",
            TextStyle {
                color: Color::WHITE,
                font_size: 16.0,
                font: uiassets.font_regular.clone(),
            },
            Default::default()
        ),
        ..Default::default()
    }).id();

    let txt_pct = commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            "0%",
            TextStyle {
                color: Color::WHITE,
                font_size: 64.0,
                font: uiassets.font_regular.clone(),
            },
            Default::default()
        ),
        ..Default::default()
    }).insert(LoadingPctText).id();

    commands.entity(inner).push_children(&[txt_loading]);
    commands.entity(inner_pct).push_children(&[txt_pct]);
    commands.entity(top).push_children(&[inner, inner_pct]);
}

fn update_loading_pct(
    mut q: Query<&mut Text, With<LoadingPctText>>,
    progress: Res<ProgressCounter>,
) {
    let progress: f32 = progress.progress().into();
    for mut txt in q.iter_mut() {
        txt.sections[0].value = format!("{:.0}%", progress * 100.0);
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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
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
    use bevy::input::ElementState;
    use bevy::input::touch::TouchPhase;

    let mut done = false;

    for ev in kbd.iter() {
        if let ElementState::Pressed = ev.state {
            done = true;
        }
    }

    for ev in mouse.iter() {
        if let ElementState::Pressed = ev.state {
            done = true;
        }
    }

    for GamepadEvent(_, kind) in gamepad.iter() {
        if let GamepadEventType::ButtonChanged(_, _) = kind {
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
