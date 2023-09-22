use crate::prelude::*;

pub struct AppStatesPlugin;

impl Plugin for AppStatesPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>();
        app.add_state::<SessionKind>();
        for state in enum_iterator::all::<AppState>() {
            app.add_systems(
                OnExit(state),
                despawn_all_recursive::<With<StateDespawnMarker>>,
            );
        }
    }
}

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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, States)]
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

/// Everything that must be despawned when transitioning the main app state
#[derive(Component)]
pub struct StateDespawnMarker;
