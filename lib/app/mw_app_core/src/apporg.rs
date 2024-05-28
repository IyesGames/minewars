use std::sync::atomic::{AtomicBool, Ordering};

use crate::{input::*, prelude::*};

pub fn plugin(app: &mut App) {
    app.init_state::<AppState>();
    for state in enum_iterator::all::<AppState>() {
        app.configure_sets(Update, InStateSet(state).run_if(in_state(state)));
    }
    app.add_plugins((
        ProgressPlugin::new(AppState::StartupLoading),
        ProgressPlugin::new(AppState::GameLoading)
            .continue_to(AppState::InGame),
    ));
    app.add_systems(
        OnExit(AppState::StartupLoading),
        despawn_all_recursive::<With<StartupLoadingCleanup>>
    );
    app.add_systems(
        OnExit(AppState::Menu),
        despawn_all_recursive::<With<MenuCleanup>>
    );
    app.add_systems(
        OnExit(AppState::GameLoading),
        despawn_all_recursive::<With<GameLoadingCleanup>>
    );
    app.add_systems(
        OnExit(AppState::InGame),
        (
            despawn_all_recursive::<With<GamePartialCleanup>>,
        )
    );
    app.add_systems(
        OnTransition { from: AppState::InGame, to: AppState::Menu },
        (
            despawn_all_recursive::<With<GameFullCleanup>>,
        )
    );
    app.init_resource::<WorkSpreader>();
    app.add_systems(First, reset_work_spreader);
}

/// State type: Which "screen" is the app in?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, States)]
#[derive(Reflect)]
#[derive(enum_iterator::Sequence)]
pub enum AppState {
    /// Initial loading screen at startup
    #[default]
    StartupLoading,
    /// Menu Screens (no game session in progress)
    Menu,
    /// Session Setup
    GameLoading,
    /// Gameplay (active session)
    InGame,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, SystemSet)]
pub struct InStateSet<S: States>(pub S);

/// Everything to despawn when exiting the startup loading state
#[derive(Component, Default)]
pub struct StartupLoadingCleanup;

/// Everything to despawn when exiting the menu state
#[derive(Component, Default)]
pub struct MenuCleanup;

/// Everything to despawn when exiting the game loading state
#[derive(Component, Default)]
pub struct GameLoadingCleanup;

/// Everything to despawn when exiting the in-game state
/// (do not use for things that should be preserved if we
/// immediately enter another session (game_loading->in_game))
#[derive(Component, Default)]
pub struct GamePartialCleanup;

/// Everything to despawn when exiting completely (no new session)
#[derive(Component, Default)]
pub struct GameFullCleanup;

#[derive(Resource, Default)]
pub struct WorkSpreader(AtomicBool);

impl WorkSpreader {
    pub fn reset(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
    pub fn acquire(&self) -> bool {
        self.0.swap(true, Ordering::SeqCst)
    }
}

fn reset_work_spreader(
    spreader: Res<WorkSpreader>,
) {
    spreader.reset();
}
