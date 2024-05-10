use bevy::{asset::LoadState, input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::MouseButtonInput}};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, load_splash_assets);
    app.add_systems(OnEnter(AppState::StartupLoading), (
        setup_splash_screen_layout,
    ));
    app.add_systems(Update, (
        splash_skip,
        splash_fade,
        spawn_splash_images,
    ).in_set(InStateSet(AppState::StartupLoading)));
    app.add_systems(
        Last,
        update_loading_pct
            .after(TrackedProgressSet)
            .run_if(in_state(AppState::StartupLoading)),
    );
}

#[derive(Resource)]
struct SplashAssets {
    iyes_logo: Handle<Image>,
    iyes_text: Handle<Image>,
    iyes_audio: Handle<AudioSource>,
    bevy_logo: Handle<Image>,
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
            timer_wait: Timer::from_seconds(wait, TimerMode::Once),
            timer_intro: Timer::from_seconds(intro, TimerMode::Once),
            timer_on: Timer::from_seconds(on, TimerMode::Once),
            timer_fade: Timer::from_seconds(fade, TimerMode::Once),
        }
    }
}

#[derive(Component)]
struct LoadingProgressIndicator;
#[derive(Component)]
struct LoadingProgressIndicatorOuter;

fn load_splash_assets(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    commands.insert_resource(SplashAssets {
        iyes_audio: ass.load("splash/iyes.flac"),
        iyes_logo: ass.load("splash/iyes_logo.png"),
        iyes_text: ass.load("splash/iyes_text.png"),
        bevy_logo: ass.load("splash/bevy.png"),
    });
}

fn setup_splash_screen_layout(
    mut commands: Commands,
) {
    commands.spawn((
        StartupLoadingCleanup,
        Camera2dBundle::default(),
    ));
    let root = commands.spawn((
        StartupLoadingCleanup,
        NodeBundle {
            style: Style {
                height: Val::Px(64.0),
                position_type: PositionType::Absolute,
                top: Val::Auto,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        }
    )).id();
    let bar_outer = commands
        .spawn((
            LoadingProgressIndicatorOuter,
            NodeBundle {
                background_color: BackgroundColor(Color::DARK_GRAY),
                style: Style {
                    width: Val::Percent(75.0),
                    height: Val::Percent(50.0),
                    padding: UiRect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id();
    let bar_inner = commands
        .spawn((
            LoadingProgressIndicator,
            NodeBundle {
                background_color: BackgroundColor(Color::GRAY),
                style: Style {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id();

    commands.entity(bar_outer).push_children(&[bar_inner]);
    commands.entity(root).push_children(&[bar_outer]);
}

fn spawn_splash_images(
    mut commands: Commands,
    spl: Option<Res<SplashAssets>>,
    ass: Res<AssetServer>,
) {
    let Some(spl) = spl else {
        return;
    };
    if ![
        spl.iyes_logo.id().untyped(),
        spl.iyes_text.id().untyped(),
        spl.iyes_audio.id().untyped(),
        spl.bevy_logo.id().untyped(),
    ].iter().all(|id| ass.load_state(*id) == LoadState::Loaded) {
        return;
    }
    commands.spawn((
        StartupLoadingCleanup,
        SpriteBundle {
            texture: spl.iyes_logo.clone(),
            transform: Transform::from_xyz(0.0, 75.0, 0.0),
            ..Default::default()
        },
        SplashFade::new(0.0, 0.0, 1.25, 1.5),
    ));
    commands.spawn((
        StartupLoadingCleanup,
        SpriteBundle {
            texture: spl.iyes_text.clone(),
            transform: Transform::from_xyz(0.0, -175.0, 0.0),
            ..Default::default()
        },
        SplashFade::new(0.25, 0.75, 0.25, 1.75),
    ));
    commands.spawn((
        StartupLoadingCleanup,
        SpriteBundle {
            texture: spl.bevy_logo.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        SplashFade::new(3.0, 0.5, 1.0, 1.5),
    ));
    commands.spawn((
        AudioBundle {
            source: spl.iyes_audio.clone(),
            settings: PlaybackSettings::DESPAWN,
            ..Default::default()
        },
    ));
    commands.remove_resource::<SplashAssets>()
}

fn update_loading_pct(
    mut commands: Commands,
    mut q_inner: Query<&mut Style, With<LoadingProgressIndicator>>,
    q_outer: Query<Entity, With<LoadingProgressIndicatorOuter>>,
    progress: Res<ProgressCounter>,
) {
    let progress: f32 = progress.progress().into();
    for mut style in &mut q_inner {
        style.width = Val::Percent(progress * 100.0);
    }
    if progress >= 1.0 {
        for e in &q_outer {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn splash_fade(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Sprite, &mut SplashFade)>,
    t: Res<Time>,
) {
    for (e, mut sprite, mut fade) in q.iter_mut() {
        if fade.timer_wait.duration().as_secs_f32() > 0.0 && !fade.timer_wait.finished() {
            fade.timer_wait.tick(t.delta());
            sprite.color.set_a(0.0);
        } else if fade.timer_intro.duration().as_secs_f32() > 0.0 && !fade.timer_intro.finished() {
            fade.timer_intro.tick(t.delta());
            let remain = fade.timer_intro.fraction();
            sprite.color.set_a(remain);
        } else if !fade.timer_on.finished() {
            fade.timer_on.tick(t.delta());
            sprite.color.set_a(1.0);
        } else if !fade.timer_fade.finished() {
            fade.timer_fade.tick(t.delta());
            let remain = fade.timer_fade.fraction_remaining();
            sprite.color.set_a(remain);
        } else {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn splash_skip(
    mut next_state: ResMut<NextState<AppState>>,
    mut kbd: EventReader<KeyboardInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut gamepad: EventReader<GamepadEvent>,
    mut touch: EventReader<TouchInput>,
    q_bar: Query<(), With<LoadingProgressIndicator>>,
    q_splash: Query<(), With<SplashFade>>,
    spl: Option<Res<SplashAssets>>,
) {
    use bevy::input::touch::TouchPhase;
    use bevy::input::ButtonState;

    if !q_bar.is_empty() || spl.is_some() {
        return;
    }

    let mut done = false;

    if q_splash.is_empty() {
        done = true;
    }

    for ev in kbd.read() {
        if let ButtonState::Pressed = ev.state {
            done = true;
        }
    }

    for ev in mouse.read() {
        if let ButtonState::Pressed = ev.state {
            done = true;
        }
    }

    for ev in gamepad.read() {
        if let GamepadEvent::Button(_) = ev {
            done = true;
        }
    }

    for ev in touch.read() {
        if let TouchPhase::Started = ev.phase {
            done = true;
        }
    }

    if done {
        next_state.set(AppState::Menu);
    }
}
