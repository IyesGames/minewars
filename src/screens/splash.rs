use crate::prelude::*;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    ecs::{schedule::SystemConfig, system::BoxedSystem},
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput, mouse::MouseButtonInput},
};

use crate::assets::SplashAssets;

pub struct SplashesPlugin;

impl Plugin for SplashesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SplashscreenPlugin {
            state: AppState::SplashIyes,
            next: AppState::SplashBevy,
            skip_to: AppState::MainMenu,
        });
        app.add_system(splash_init_iyes.in_schedule(OnEnter(AppState::SplashIyes)));
        app.add_plugin(SplashscreenPlugin {
            state: AppState::SplashBevy,
            next: AppState::MainMenu,
            skip_to: AppState::MainMenu,
        });
        app.add_system(splash_init_bevy.in_schedule(OnEnter(AppState::SplashBevy)));
        app.add_system(remove_resource::<SplashAssets>.in_schedule(OnEnter(AppState::MainMenu)));
    }
}

struct SplashscreenPlugin<S: States> {
    state: S,
    next: S,
    skip_to: S,
}

impl<S: States> Plugin for SplashscreenPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_system(
            setup_splashscreen(self.next.clone()).in_schedule(OnEnter(self.state.clone())),
        );
        app.add_system(
            despawn_all_recursive::<With<SplashCleanup>>.in_schedule(OnExit(self.state.clone())),
        );
        app.add_system(remove_resource::<SplashNext<S>>.in_schedule(OnExit(self.state.clone())));
        app.add_system(splash_fade::<S>.run_if(in_state(self.state.clone())));
        app.add_system(splash_skip(self.skip_to.clone()).run_if(in_state(self.state.clone())));
    }
    fn is_unique(&self) -> bool {
        false
    }
}

fn setup_splashscreen<S: States>(next: S) -> SystemConfig {
    let next = next.clone();
    let system = move |mut commands: Commands| {
        commands.insert_resource(SplashNext(next.clone()));
        commands.spawn((
            SplashCleanup,
            Camera2dBundle {
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                },
                ..Default::default()
            },
        ));
    };
    let boxed: BoxedSystem = Box::new(IntoSystem::into_system(system));
    IntoSystemConfig::into_config(boxed)
}

fn splash_init_iyes(mut commands: Commands, splashes: Res<SplashAssets>) {
    commands.spawn((
        SplashCleanup,
        SpriteBundle {
            texture: splashes.iyes_logo.clone(),
            transform: Transform::from_xyz(0.0, 75.0, 0.0),
            ..Default::default()
        },
        SplashFade::new(0.0, 0.0, 1.25, 1.5),
    ));
    commands.spawn((
        SplashCleanup,
        SpriteBundle {
            texture: splashes.iyes_text.clone(),
            transform: Transform::from_xyz(0.0, -175.0, 0.0),
            ..Default::default()
        },
        SplashFade::new(0.25, 0.75, 0.25, 1.75),
    ));
}

fn splash_init_bevy(mut commands: Commands, splashes: Res<SplashAssets>) {
    commands.spawn((
        SplashCleanup,
        SpriteBundle {
            texture: splashes.bevy.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        SplashFade::new(0.0, 0.5, 1.0, 1.5),
    ));
}

#[derive(Component)]
struct SplashCleanup;

#[derive(Resource)]
struct SplashNext<S: States>(S);

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

fn splash_fade<S: States>(
    mut q: Query<(&mut Sprite, &mut SplashFade)>,
    mut next_state: ResMut<NextState<S>>,
    t: Res<Time>,
    next: Res<SplashNext<S>>,
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
        next_state.set(next.0.clone());
    }
}

fn splash_skip<S: States>(skip_to: S) -> SystemConfig {
    let skip_to = skip_to.clone();
    let system = move |mut next_state: ResMut<NextState<S>>,
                       mut kbd: EventReader<KeyboardInput>,
                       mut mouse: EventReader<MouseButtonInput>,
                       mut gamepad: EventReader<GamepadEvent>,
                       mut touch: EventReader<TouchInput>| {
        use bevy::input::touch::TouchPhase;
        use bevy::input::ButtonState;

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
            if let GamepadEvent::Button(_) = ev {
                done = true;
            }
        }

        for ev in touch.iter() {
            if let TouchPhase::Started = ev.phase {
                done = true;
            }
        }

        if done {
            next_state.set(skip_to.clone());
        }
    };
    let boxed: BoxedSystem = Box::new(IntoSystem::into_system(system));
    IntoSystemConfig::into_config(boxed)
}
