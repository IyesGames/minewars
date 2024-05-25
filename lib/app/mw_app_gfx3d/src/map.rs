use mw_common::{game::{StructureKind, TileKind}, grid::{Coord, Hex}};
use mw_app_core::map::{MapTileIndex, tile::{MwTilePos, TileGent}};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

#[derive(Component, Debug)]
pub struct TileAss3d {
    /// What 3D Asset Tile Kind is the tile entity using?
    pub kind: Ass3dTileKind,
    /// What TileSet mode variant?
    pub variant: Ass3dTileVariant,
    /// Rotation to use
    pub rotation: u8,
    /// Neighbors for TileSet (variant and rotation are derived from this)
    pub neighmask: u8,
    /// Keep track of which alternative (if multiple are available) model
    /// was randomly selected for each LOD, for consistency when switching LODs.
    pub subvariant: [u8; 3],
}

impl TileAss3d {
    pub fn rotation_quat(&self) -> Quat {
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_3 * self.rotation as f32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ass3dTileKind {
    BadTile,
    Water,
    Regular,
    Fertile,
    Forest,
    Mountain,
    Destroyed,
    HarvestedFertile,
    HarvestedRegular,
    HarvestedForest,
    HarvestedMountain,
    Road,
    Cit,
    Tower,
    WallMid,
    Bridge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ass3dTileVariant {
    V0,
    V1,
    V2A,
    V2B,
    V2C,
    V3A,
    V3B,
    V3C,
    V3D,
    V4A,
    V4B,
    V4C,
    V5,
    V6,
}

pub fn compute_tile_ass3d(
    q_map: Query<&MapTileIndex>,
    mut q_tile: Query<
        (&mut TileAss3d, &MwTilePos, &TileKind, Option<&TileGent>),
        Or<(Changed<TileKind>, Changed<TileGent>)>,
    >,
    q_tilekind: Query<&TileKind>,
) {
    let mut rng = thread_rng();
    let tile_index = q_map.single();
    for (mut ass3d, tilepos, kind, gent) in &mut q_tile {
        let ass3d_kind = match (kind, gent) {
            (_, Some(TileGent::Cit(_))) => Ass3dTileKind::Cit,
            (_, Some(TileGent::Structure(StructureKind::Barricade))) => Ass3dTileKind::WallMid,
            (_, Some(TileGent::Structure(StructureKind::Bridge))) => Ass3dTileKind::Bridge,
            (_, Some(TileGent::Structure(StructureKind::WatchTower))) => Ass3dTileKind::Tower,
            (TileKind::Water, _) => Ass3dTileKind::Water,
            (TileKind::Fertile, _) => Ass3dTileKind::Fertile,
            (TileKind::Destroyed, _) => Ass3dTileKind::Destroyed,
            (TileKind::Mountain, _) => Ass3dTileKind::Mountain,
            (TileKind::Forest, _) => Ass3dTileKind::Forest,
            (TileKind::FoundationRoad, _) => Ass3dTileKind::Road,
            (TileKind::Regular, _) => Ass3dTileKind::Regular,
            _ => Ass3dTileKind::BadTile,
        };
        let mut neighs = [TileKind::Water; 6];
        for (i, cc) in Hex::from(tilepos.0).iter_n1().enumerate() {
            if let Some(kind) = tile_index.0.get(cc.into()).and_then(|ee| q_tilekind.get(*ee).ok()) {
                neighs[i] = *kind;
            }
        }
        let neighmask = match ass3d_kind {
            Ass3dTileKind::Water =>
                tile_neighmask(neighs, |k| k == TileKind::Water),
            Ass3dTileKind::Regular =>
                tile_neighmask(neighs, |k| k != TileKind::Water && k != TileKind::Mountain),
            Ass3dTileKind::Fertile =>
                tile_neighmask(neighs, |k| k != TileKind::Water),
            Ass3dTileKind::Forest =>
                tile_neighmask(neighs, |k| k == TileKind::Forest),
            Ass3dTileKind::Mountain =>
                tile_neighmask(neighs, |k| k == TileKind::Mountain),
            Ass3dTileKind::Destroyed =>
                tile_neighmask(neighs, |k| k != TileKind::Water),
            Ass3dTileKind::HarvestedFertile =>
                tile_neighmask(neighs, |k| k != TileKind::Water),
            Ass3dTileKind::HarvestedRegular =>
                tile_neighmask(neighs, |k| k != TileKind::Water && k != TileKind::Mountain),
            Ass3dTileKind::HarvestedForest =>
                tile_neighmask(neighs, |k| k == TileKind::Forest),
            Ass3dTileKind::HarvestedMountain =>
                tile_neighmask(neighs, |k| k == TileKind::Mountain),
            Ass3dTileKind::Road =>
                tile_neighmask(neighs, |k| k == TileKind::FoundationRoad),
            Ass3dTileKind::Bridge =>
                tile_neighmask(neighs, |k| k != TileKind::Water && k != TileKind::Mountain && k != TileKind::Forest),
            Ass3dTileKind::Cit => 0,
            Ass3dTileKind::Tower => 0,
            Ass3dTileKind::WallMid => 0,
            Ass3dTileKind::BadTile => 0,
        };
        let (variant, mut allowed_rotations) = crate::misc::TILESET_VARIANT_LUT[neighmask as usize];
        let n_allowed_rotations = allowed_rotations.count_ones();
        let mut rotation_random_i = rng.gen_range(0..n_allowed_rotations);
        let mut rotation = 0;
        for ri in 0..6 {
            if allowed_rotations == 0 {
                break;
            }
            if allowed_rotations & 1 != 0 {
                rotation = ri;
                if rotation_random_i > 0 {
                    rotation_random_i -= 1;
                } else {
                    break;
                }
            }
            allowed_rotations >>= 1;
        }
        *ass3d = TileAss3d {
            kind: ass3d_kind,
            variant,
            rotation,
            neighmask,
            subvariant: rng.gen(),
        };
    }
}

fn tile_neighmask(
    neighs: [TileKind; 6],
    f_pred: impl Fn(TileKind) -> bool,
) -> u8 {
    let mut r = 0;
    for i in 0..6 {
        if f_pred(neighs[i]) {
            r |= 1 << i;
        }
    }
    r
}
