use crate::prelude::*;

pub struct AppStatesPlugin;

impl Plugin for AppStatesPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameMode>();
        app.add_state::<AppState>();
        app.add_state::<SessionKind>();
        for state in enum_iterator::all::<AppState>() {
            app.configure_sets(Update, InStateSet(state).run_if(in_state(state)));
            app.add_systems(
                OnExit(state),
                despawn_all_recursive::<With<StateDespawnMarker>>,
            );
        }
        for state in enum_iterator::all::<SessionKind>() {
            app.configure_sets(Update, InStateSet(state).run_if(in_state(state)));
        }
        for state in enum_iterator::all::<GameMode>() {
            app.configure_sets(Update, InStateSet(state).run_if(in_state(state)));
            app.configure_sets(Update, InGameSet(Some(state)).in_set(InStateSet(state)).in_set(InStateSet(AppState::InGame)));
        }
        app.configure_sets(Update, InGameSet(None).in_set(InStateSet(AppState::InGame)));
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, SystemSet)]
pub struct InStateSet<S: States>(pub S);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, SystemSet)]
pub struct InGameSet(pub Option<GameMode>);

/// State type: Which "screen" is the app in?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, States)]
#[derive(Reflect)]
#[derive(enum_iterator::Sequence)]
pub enum AppState {
    /// Initial loading screen at startup
    #[default]
    AssetsLoading,
    /// Splash with the IyesGames logo
    SplashIyes,
    /// Splash with the Bevy logo
    SplashBevy,
    /// Main Menu
    MainMenu,
    /// Gameplay
    InGame,
    /// Map Editor
    Editor,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, States)]
#[derive(Reflect)]
#[derive(enum_iterator::Sequence)]
/// State type: What drives the game? Where do data and events come from?
pub enum SessionKind {
    /// Nowhere. We are not in a game session of any sort.
    #[default]
    Disconnected,
    /// We are connected to a Host server. Network protocol drives the game.
    NetHost,
    /// We host/run our own gameplay in Bevy. The BevyHost drives the game.
    BevyHost,
    /// We are playing a replay file. We read data from it.
    File,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, States)]
#[derive(Reflect)]
#[derive(enum_iterator::Sequence)]
pub enum GameMode {
    #[default]
    NoGame,
    Minesweeper,
    Minewars,
}

/// Everything that must be despawned when transitioning the main app state
#[derive(Component)]
pub struct StateDespawnMarker;
