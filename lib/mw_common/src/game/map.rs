//! Stuff for working with MineWars Game Map Data

use crate::game::{MapTileInit, MapDataInit, TileKind};
use crate::grid::{Pos, Coord};
use crate::grid::map::{CompactMapCoordExt, MapData};
use crate::algo;
use crate::plid::PlayerId;

/// Compute "fertile land" areas
///
/// The initial map data is provided (by worldgen, proto, etc) with
/// only base geographical data (water/land/mountain).
///
/// The tiles to be converted into "fertile land" are not stored and need to be
/// computed (both on the client and server) and assumed to match / be deterministic.
pub fn fertilize_land<C: CompactMapCoordExt> (
    map: &mut MapData<C, MapTileInit>,
    fertile_depth: u8,
    _q: &mut algo::FloodQ,
) {
    let sz = map.size();

    // Any land around water becomes Fertile
    for r in (1..sz).rev() {
        let mut cnt = 0;
        for c in C::origin().iter_ring(r) {
            if map[c].kind == TileKind::Water { continue; }

            for c2 in c.iter_ring(fertile_depth) {
                if c2.ring() <= sz && map[c2].kind == TileKind::Water {
                    map[c].kind = TileKind::Fertile;
                    cnt += 1;
                    break;
                }
            }
        }
        if cnt == 0 {
            break;
        }
    }

    // Detect areas entirely surrounded by fertile land and fill them
    // (disabled) I don't like the huge fertile areas this results in

    /*
    // floodfill from center to mark inner regular land
    q.push_back(C::origin().into());
    algo::flood::<C, _>(q, |c, _| {
        if c.ring() <= sz && !map[c].mark && map[c].kind == TileKind::Regular {
            map[c].mark = true;
            algo::FloodSelect::Yes
        } else {
            algo::FloodSelect::No
        }
    });

    // replace unmarked regular land with fertile land
    for c in map.data_mut() {
        if c.mark {
            c.mark = false;
        } else if c.kind == TileKind::Regular {
            c.kind = TileKind::Fertile;
        }
    }
    */
}

/*
/// Compute region affiliations
pub fn gen_regions<C: CompactMapCoordExt + Coord>(
    gen: &mut MapDataInit<C>,
    q: &mut algo::FloodQ,
) {
    use crate::algo::FloodSelect;

    q.clear();

    // use combined floodfill starting from all the cit locations
    for &b in gen.cits.iter() {
        q.push_back(b.into());
    }

    algo::flood(q, |c, s| {
        // want to ensure that mountain clusters do not get divided
        // between multiple regions

        // do not expand out from mountains into land
        if gen.map[s].kind == TileKind::Mountain && gen.map[c].kind != TileKind::Mountain {
            return FloodSelect::No;
        }

        // region we are coming from
        let reg = gen.map[s].region;

        if gen.map[c].kind != TileKind::Water && gen.map[c].region == 0xFF {
            // cell needs to be assigned a region; copy source region
            gen.map[c].region = reg;

            // mountains skip the queue to be expanded first
            // (ensuring they are entirely in the current region)
            if gen.map[c].kind == TileKind::Mountain {
                FloodSelect::YesPrio
            } else {
                FloodSelect::Yes
            }
        } else {
            FloodSelect::No
        }
    });
}
*/

#[inline(always)]
pub fn compute_fog_of_war<C: Coord>(
    vis_radius: u8,
    queue_tmp_dirty: &mut Vec<C>,
    my_plid: PlayerId,
    tiles: impl IntoIterator<Item = C>,
    fetch_owner_at: impl Fn(C) -> Option<PlayerId>,
    mut set_vis_at: impl FnMut(C, bool),
) {
    queue_tmp_dirty.clear();
    // PERF: find more ways to save on the number of tiles to recompute
    for c in tiles {
        if let Some(c_owner) = fetch_owner_at(c) {
            if c_owner == my_plid {
                // we GAINED ownership
                // grant visibility of the tile and all its neighbors
                set_vis_at(c, true);
                for r in 1..=vis_radius {
                    for c2 in c.iter_ring(r) {
                        set_vis_at(c2, true);
                    }
                }
            } else {
                // we LOST ownership
                // the tile and all its neighbors may or may not lose visibility
                // must test them all individually; queue coords for testing
                queue_tmp_dirty.push(c);
                for r in 1..=vis_radius {
                    queue_tmp_dirty.extend(c.iter_ring(r));
                }
            }
        }
    }

    if queue_tmp_dirty.is_empty() {
        return;
    }

    queue_tmp_dirty.sort_unstable();
    queue_tmp_dirty.dedup();

    for c in queue_tmp_dirty.drain(..) {
        let mut vis = false;
        if let Some(c_owner) = fetch_owner_at(c) {
            if c_owner == my_plid {
                vis = true;
            } else {
                'outer: for r in 1..=vis_radius {
                    for c2 in c.iter_ring(r) {
                        if let Some(c2_owner) = fetch_owner_at(c2) {
                            if c2_owner == my_plid {
                                vis = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }
            set_vis_at(c, vis);
        }
    }
}
