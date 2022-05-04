use crate::assets::TileAssets;
use crate::prelude::*;
use crate::AppGlobalState;
use mw_common::grid::map::CompactMapCoordExt;
use mw_common::{RoadState, MineKind};
use mw_common::plid::PlayerId;
use mw_common::grid::*;

use self::tileid::CoordTileids;

pub enum MapLabels {
    /// Anything that sends MapEvents should come before this
    ApplyEvents,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MapEvent>();
        app.add_enter_system(AppGlobalState::InGame, setup_map);
        app.add_exit_system(AppGlobalState::InGame, despawn_with_recursive::<MapCleanup>);
    }
}

#[derive(Component)]
struct MapCleanup;

pub struct MapDescriptor {
    pub size: u8,
    pub topology: Topology,
}

pub enum MapEventKind {
    Owner {
        plid: PlayerId,
    },
    Digit {
        digit: u8,
    },
    Mine {
        kind: Option<MineKind>,
    },
    Road {
        state: Option<RoadState>,
    },
    Explosion {
        kind: MineKind,
    },
    ActiveMine,
}

pub struct MapEvent {
    c: Pos,
    kind: MapEventKind,
}

fn setup_map(
    mut commands: Commands,
    descriptor: Res<MapDescriptor>,
    tiles: Res<TileAssets>,
) {
    match descriptor.topology {
        Topology::Hex => setup_map_topology::<Hex>(&mut commands, &*descriptor, &*tiles),
        Topology::Sq => setup_map_topology::<Sq>(&mut commands, &*descriptor, &*tiles),
        _ => unimplemented!(),
    }
}

fn setup_map_topology<C: CoordTileids + CompactMapCoordExt>(
    commands: &mut Commands,
    descriptor: &MapDescriptor,
    tiles: &TileAssets,
) {
    for c in C::iter_coords(descriptor.size) {
        let pos = c.translation() * C::TILE_OFFSET;
        commands.spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: C::TILEID_LAND,
                ..Default::default()
            },
            texture_atlas: tiles.atlas.clone(),
            transform: Transform::from_translation(pos.extend(0.0)),
            ..Default::default()
        }).insert(MapCleanup);
    }
}

mod tileid {
    #![allow(dead_code)]

    use crate::prelude::*;
    use bevy::math::const_vec2;
    use mw_common::grid::*;

    use crate::assets::TILESZ;

    pub trait CoordTileids: Coord {
        const TILE_OFFSET: Vec2;
        const TILEID_LAND: usize;
        const TILEID_CURSOR: usize;
        const TILEID_ROADS: &'static [usize];
    }

    impl CoordTileids for Hex {
        const TILE_OFFSET: Vec2 = const_vec2!([224.0, 256.0]);
        const TILEID_LAND: usize = 0o1;
        const TILEID_CURSOR: usize = 0o0;
        const TILEID_ROADS: &'static [usize] = &[0o60, 0o61, 0o62, 0o63, 0o64, 0o65];
    }

    impl CoordTileids for Sq {
        const TILE_OFFSET: Vec2 = const_vec2!([224.0, 224.0]);
        const TILEID_LAND: usize = 0o11;
        const TILEID_CURSOR: usize = 0o10;
        const TILEID_ROADS: &'static [usize] = &[0o70, 0o71, 0o72, 0o73];
    }

    pub const ITEM_MINE: usize = 0o4;
    pub const ITEM_DECOY: usize = 0o5;
    pub const EXPLODE_MINE: usize = 0o14;
    pub const EXPLODE_DECOY: usize = 0o15;
    pub const MINE_ACTIVE: usize = 0o24;

    pub const GEO_WATER: usize = 0o20;
    pub const GEO_FERTILE: usize = 0o21;
    pub const GEO_MOUNTAIN: usize = 0o20;

    pub const LANDMARK_CITY: usize = 0o40;
    pub const LANDMARK_TOWER: usize = 0o41;
    pub const DECAL_SKULL: usize = 0o50;

    pub const DIGITS: [usize; 8] = [0, 0o51, 0o52, 0o53, 0o54, 0o55, 0o56, 0o57];
}
