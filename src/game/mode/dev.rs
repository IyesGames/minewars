use crate::prelude::*;

use crate::camera::GridCursor;
use crate::game::skip_lobby_state;
use crate::map::{MapEvent, MapEventKind, MapLabels};

use mw_common::game::MineKind;
use mw_common::app::ActivePlid;

pub struct GameModeDevPlugin;

impl Plugin for GameModeDevPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Dev)
        );
        app.add_system(
            debug_mapevents
                .run_in_state(AppGlobalState::InGame)
                .run_in_state(GameMode::Dev)
                .after("cursor")
                .before(MapLabels::ApplyEvents)
        );
    }
}

fn debug_mapevents(
    mut evw_map: EventWriter<MapEvent>,
    crs: Res<GridCursor>,
    input: Res<Input<MouseButton>>,
    my_plid: Res<ActivePlid>,
) {
    if input.just_pressed(MouseButton::Middle) {
        let mut rng = thread_rng();

        let kind = if rng.gen_bool(0.5) {
            MineKind::Mine
        } else {
            MineKind::Decoy
        };

        let owner = rng.gen_range(0u8..7u8);
        let digit = rng.gen_range(0u8..8u8);

        evw_map.send(MapEvent {
            c: crs.0,
            plid: my_plid.0,
            kind: MapEventKind::Explosion { kind },
        });

        evw_map.send(MapEvent {
            c: crs.0,
            plid: my_plid.0,
            kind: MapEventKind::Owner { plid: owner.into() },
        });

        evw_map.send(MapEvent {
            c: crs.0,
            plid: my_plid.0,
            kind: MapEventKind::Digit { digit },
        });
    }
}
