use mw_app_core::map::{cit::*, tile::*, *};
use mw_common::plid::PlayerId;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        setup_cit_entities
            .track_progress(),
    )
        .in_set(InStateSet(AppState::GameLoading))
        .in_set(NeedsMapGovernorSet),
    );
}

fn setup_cit_entities(
    mut commands: Commands,
    spreader: Res<WorkSpreader>,
    q_map: Query<(Entity, &MapDataOrig, &MapTileIndex, Has<CitIndex>), With<MapGovernor>>,
) -> Progress {
    let (e_map, map_src, tile_index) = match q_map.get_single() {
        Err(_) => return false.into(),
        Ok((_, _, _, true)) => return true.into(),
        Ok((e, orig, tile_index, false)) => (e, orig, tile_index),
    };
    if spreader.acquire() {
        return false.into();
    }

    let mut cit_index = CitIndex {
        by_id: Vec::with_capacity(map_src.cits.len()),
        by_pos: HashMap::with_capacity(map_src.cits.len()),
    };

    for (i, cit_pos) in map_src.cits.iter().enumerate() {
        let cit_pos = *cit_pos;
        let e_cit = commands.spawn(
            CitBundle {
                cleanup: GamePartialCleanup,
                marker: MwCit,
                region: CitRegion(i as u8),
                owner: CitOwner::Plid(PlayerId::Neutral),
                economy: CitEconomy {
                    money: 0,
                    income: 0,
                    res: 0,
                    export: 255,
                    import: 255,
                },
            },
        ).id();
        cit_index.by_id.push(e_cit);
        cit_index.by_pos.insert(cit_pos, e_cit);
        commands.entity(tile_index.0[(cit_pos).into()])
            .insert(TileGent::Cit(i as u8));
    }

    commands.entity(e_map)
        .insert(cit_index);

    false.into()
}
