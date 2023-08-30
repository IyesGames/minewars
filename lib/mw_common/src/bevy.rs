use crate::{prelude::*, grid::Topology, game::MapDescriptor};
use iyes_bevy_extras::prelude::*;

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

/// Everything that must be despawned when transitioning the main app state
#[derive(Component)]
pub struct StateDespawnMarker;

pub fn map_topology_is(topo: Topology) -> impl FnMut(Option<Res<MapDescriptor>>) -> bool {
    move |desc: Option<Res<MapDescriptor>>| {
        desc.map(|desc| desc.topology == topo).unwrap_or(false)
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapTopologySet(Topology);

pub struct MwCommonPlugin;

impl Plugin for MwCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>();
        for state in enum_iterator::all::<AppState>() {
            app.add_systems(
                OnExit(state),
                despawn_all_recursive::<With<StateDespawnMarker>>,
            );
        }
        for topo in enum_iterator::all::<Topology>() {
            app.configure_set(Update, MapTopologySet(topo).run_if(map_topology_is(topo)));
        }
    }
}
