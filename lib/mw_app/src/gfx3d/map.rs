use mw_common::{game::{StructureKind, TileKind}, grid::{Coord, Hex}};

use crate::prelude::*;
use crate::map::{MapTileIndex, MwTilePos, TileGent};

use super::*;

pub struct Gfx3dMapPlugin;

impl Plugin for Gfx3dMapPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, (
        // //     compute_tile_ass3d
        // //         .in_set(InGameSet(None))
        // //         .in_set(Gfx3dSet::Any),
        // ));
    }
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
    tile_index: Res<MapTileIndex<Hex>>,
    mut q_tile: Query<
        (&mut TileAss3d, &MwTilePos, &TileKind, Option<&TileGent>),
        Or<(Changed<TileKind>, Changed<TileGent>)>,
    >,
    q_tilekind: Query<&TileKind>,
) {
    let mut rng = thread_rng();
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
            if let Some(kind) = tile_index.0.get(cc).and_then(|ee| q_tilekind.get(*ee).ok()) {
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
        let (variant, mut allowed_rotations) = TILESET_VARIANT_LUT[neighmask as usize];
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

/// LUT for selecting which tile 3D asset to use for a given tile.
/// Indicates what asset to use and a mask of allowed rotations.
const TILESET_VARIANT_LUT: &[(Ass3dTileVariant, u8)] = &[
    (Ass3dTileVariant::V0 , 0b111111), // 000000
    (Ass3dTileVariant::V1 , 0b000001), // 000001
    (Ass3dTileVariant::V1 , 0b000010), // 000010
    (Ass3dTileVariant::V2A, 0b000001), // 000011
    (Ass3dTileVariant::V1 , 0b000100), // 000100
    (Ass3dTileVariant::V2B, 0b000001), // 000101
    (Ass3dTileVariant::V2A, 0b000010), // 000110
    (Ass3dTileVariant::V3A, 0b000001), // 000111
    (Ass3dTileVariant::V1 , 0b001000), // 001000
    (Ass3dTileVariant::V2C, 0b001001), // 001001
    (Ass3dTileVariant::V2B, 0b000010), // 001010
    (Ass3dTileVariant::V3B, 0b000001), // 001011
    (Ass3dTileVariant::V2A, 0b000100), // 001100
    (Ass3dTileVariant::V3C, 0b000100), // 001101
    (Ass3dTileVariant::V3A, 0b000010), // 001110
    (Ass3dTileVariant::V4A, 0b000001), // 001111
    (Ass3dTileVariant::V1 , 0b010000), // 010000
    (Ass3dTileVariant::V2B, 0b010000), // 010001
    (Ass3dTileVariant::V2C, 0b010010), // 010010
    (Ass3dTileVariant::V3C, 0b000001), // 010011
    (Ass3dTileVariant::V2B, 0b000100), // 010100
    (Ass3dTileVariant::V3D, 0b010101), // 010101
    (Ass3dTileVariant::V3B, 0b000010), // 010110
    (Ass3dTileVariant::V4B, 0b000001), // 010111
    (Ass3dTileVariant::V2A, 0b001000), // 011000
    (Ass3dTileVariant::V3B, 0b001000), // 011001
    (Ass3dTileVariant::V3C, 0b001000), // 011010
    (Ass3dTileVariant::V4C, 0b001001), // 011011
    (Ass3dTileVariant::V3A, 0b000100), // 011100
    (Ass3dTileVariant::V4B, 0b000100), // 011101
    (Ass3dTileVariant::V4A, 0b000010), // 011110
    (Ass3dTileVariant::V5 , 0b000001), // 011111
    (Ass3dTileVariant::V1 , 0b100000), // 100000
    (Ass3dTileVariant::V2A, 0b100000), // 100001
    (Ass3dTileVariant::V2B, 0b100000), // 100010
    (Ass3dTileVariant::V3A, 0b100000), // 100011
    (Ass3dTileVariant::V2C, 0b100100), // 100100
    (Ass3dTileVariant::V3B, 0b100000), // 100101
    (Ass3dTileVariant::V3C, 0b000010), // 100110
    (Ass3dTileVariant::V4A, 0b100000), // 100111
    (Ass3dTileVariant::V2B, 0b001000), // 101000
    (Ass3dTileVariant::V3C, 0b100000), // 101001
    (Ass3dTileVariant::V3D, 0b101010), // 101010
    (Ass3dTileVariant::V4B, 0b100000), // 101011
    (Ass3dTileVariant::V3B, 0b000100), // 101100
    (Ass3dTileVariant::V4C, 0b100100), // 101101
    (Ass3dTileVariant::V4B, 0b000010), // 101110
    (Ass3dTileVariant::V5 , 0b100000), // 101111
    (Ass3dTileVariant::V2A, 0b010000), // 110000
    (Ass3dTileVariant::V3A, 0b010000), // 110001
    (Ass3dTileVariant::V3B, 0b010000), // 110010
    (Ass3dTileVariant::V4A, 0b010000), // 110011
    (Ass3dTileVariant::V3C, 0b010000), // 110100
    (Ass3dTileVariant::V4B, 0b010000), // 110101
    (Ass3dTileVariant::V4C, 0b010010), // 110110
    (Ass3dTileVariant::V5 , 0b010000), // 110111
    (Ass3dTileVariant::V3A, 0b001000), // 111000
    (Ass3dTileVariant::V4A, 0b001000), // 111001
    (Ass3dTileVariant::V4B, 0b001000), // 111010
    (Ass3dTileVariant::V5 , 0b001000), // 111011
    (Ass3dTileVariant::V4A, 0b000100), // 111100
    (Ass3dTileVariant::V5 , 0b000100), // 111101
    (Ass3dTileVariant::V5 , 0b000010), // 111110
    (Ass3dTileVariant::V6 , 0b111111), // 111111
];
