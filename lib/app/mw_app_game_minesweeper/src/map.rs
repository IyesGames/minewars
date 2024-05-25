use mw_app_core::{driver::DriverGovernor, map::*};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        gen_simple_map
            .track_progress()
            .run_if(any_filter::<(With<SimpleMapGenerator>, With<DriverGovernor>)>),
    )
        .in_set(InStateSet(AppState::GameLoading)),
    );
}

/// Add this onto the Driver Governor to generate a simple
/// flat map for a local/offline session.
///
/// During the GameLoading state, this will
/// enable a system that sets up the Map Governor.
#[derive(Component)]
pub struct SimpleMapGenerator {
    pub topology: Topology,
    pub size: u8,
}

fn gen_simple_map(
    mut commands: Commands,
    q_map: Query<(), With<MapGovernor>>,
    q_driver: Query<&SimpleMapGenerator, With<DriverGovernor>>,
) -> Progress {
    if !q_map.is_empty() {
        return true.into();
    }
    let gen = q_driver.single();
    let mut empty_tile = MapTileDataOrig::default();
    empty_tile.set_kind(TileKind::Regular);
    empty_tile.set_item(ItemKind::Safe);
    empty_tile.set_region(0);
    let map_src = MapDataPos::new(gen.size, empty_tile);

    commands.spawn((
        MapGovernorBundle {
            cleanup: GameFullCleanup,
            marker: MapGovernor,
            desc: MapDescriptor {
                size: gen.size,
                topology: gen.topology,
            },
            map_src: MapDataOrig {
                map: map_src,
                cits: vec![],
            },
            grid_cursor: default(),
            grid_cursor_tile_entity: default(),
        },
    ));

    // for the sake of the progress bar not appearing like it is
    // going backwards (there will be new systems on the next frame
    // after we have spawned the Map Governor),
    // return a fake large total amount
    Progress {
        done: 1,
        total: 8,
    }
}
