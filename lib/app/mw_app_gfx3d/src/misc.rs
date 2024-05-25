use crate::{prelude::*, map::Ass3dTileVariant, settings::Gfx3dImpl};

pub fn plugin(app: &mut App) {
    app.configure_sets(Update, (
        Gfx3dImplSet::Any.run_if(rc_gfx3d_any),
        Gfx3dImplSet::Simple.run_if(rc_gfx3d_simple),
        Gfx3dImplSet::Bespoke.run_if(rc_gfx3d_bespoke),
    ));
    app.configure_sets(OnEnter(AppState::InGame), (
        Gfx3dImplSet::Any.run_if(rc_gfx3d_any),
        Gfx3dImplSet::Simple.run_if(rc_gfx3d_simple),
        Gfx3dImplSet::Bespoke.run_if(rc_gfx3d_bespoke),
    ));
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gfx3dImplSet {
    Any,
    Simple,
    Bespoke,
}

fn rc_gfx3d_any(
    settings: Settings,
) -> bool {
    settings.get::<Gfx3dImpl>().map(|s|
        *s == Gfx3dImpl::Bespoke || *s == Gfx3dImpl::Simple
    ).unwrap_or(false)
}

fn rc_gfx3d_simple(
    settings: Settings,
) -> bool {
    settings.get::<Gfx3dImpl>().map(|s|
        *s == Gfx3dImpl::Simple
    ).unwrap_or(false)
}

fn rc_gfx3d_bespoke(
    settings: Settings,
) -> bool {
    settings.get::<Gfx3dImpl>().map(|s|
        *s == Gfx3dImpl::Bespoke
    ).unwrap_or(false)
}

pub(crate) const TILE_SCALE: f32 = 64.0;
pub(crate) const RENDER_RANGE: f32 = 8192.0;

/// LUT for selecting which tile 3D asset to use for a given tile.
/// Indicates what asset to use and a mask of allowed rotations.
pub(crate) const TILESET_VARIANT_LUT: &[(Ass3dTileVariant, u8)] = &[
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
