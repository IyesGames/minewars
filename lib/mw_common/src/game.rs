use enum_iterator::Sequence;
use enum_map::{Enum, EnumMap};
use num_derive::FromPrimitive;
use modular_bitfield::prelude::*;

use crate::prelude::*;
use crate::grid::*;

pub mod event;

pub type CitId = u8;

/// The possibilities of what can be on a given tile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(Enum, FromPrimitive, ToPrimitive, BitfieldSpecifier, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
#[bits = 2]
pub enum ItemKind {
    #[default]
    Safe,
    Mine,
    Decoy,
    Trap,
}

/// The base variant of a map tile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(Enum, FromPrimitive, ToPrimitive, BitfieldSpecifier, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
#[bits = 3]
pub enum TileKind {
    Water,
    Destroyed,
    #[default]
    Regular,
    Fertile,
    FoundationStruct,
    FoundationRoad,
    Forest,
    Mountain,
}

impl TileKind {
    pub const fn ownable(self) -> bool {
        match self {
            TileKind::Water => false,
            _ => true,
        }
    }

    pub const fn is_land(self) -> bool {
        match self {
            TileKind::Water |
            TileKind::Forest |
            TileKind::Mountain |
            TileKind::FoundationStruct => false,
            TileKind::Regular |
            TileKind::Fertile |
            TileKind::Destroyed |
            TileKind::FoundationRoad => true,
        }
    }

    pub const fn is_rescluster(self) -> bool {
        match self {
            TileKind::Forest |
            TileKind::Mountain => true,
            TileKind::Water |
            TileKind::Regular |
            TileKind::Fertile |
            TileKind::Destroyed |
            TileKind::FoundationRoad |
            TileKind::FoundationStruct=> false,
        }
    }

    pub const fn is_harvestable(self) -> bool {
        match self {
            TileKind::Water |
            TileKind::Destroyed |
            TileKind::FoundationRoad |
            TileKind::FoundationStruct => false,
            TileKind::Regular |
            TileKind::Fertile |
            TileKind::Forest |
            TileKind::Mountain => true,
        }
    }
}

/// Is a structure in-production or completed?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub enum ProdState {
    Pending,
    Built,
}

/// All the various structures that can be built
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(Enum, FromPrimitive, ToPrimitive, BitfieldSpecifier, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
#[bits = 2]
pub enum StructureKind {
    #[default]
    Road,
    Barricade,
    WatchTower,
    Bridge,
}

/// The gameplay actions that a player can perform on a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum, Sequence)]
pub enum ActionKind {
    Explore,
    Strike,
    Deploy,
    Harvest,
    Smoke,
    Reveal,
    Build,
    Bulldoze,
}

/// Values used by different game mechanics
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct BalancingTable {
    pub res_base: u8,
    pub res_tile: EnumMap<TileKind, u8>,
    pub res_tile_harvest: EnumMap<TileKind, u16>,
    pub cit_starting_money_spawn: u16,
    pub cit_starting_money_other: u16,
    pub mult_cancel_return: (u8, u8),
    pub mult_bulldoze_return: (u8, u8),
    pub mult_capture_item_sell: (u8, u8),
    pub mult_cost_foreign_region: (u8, u8),
    pub mult_costsharing_local_contribution: (u8, u8),
    pub mult_cit_capture_keep_money: (u8, u8),
    pub radius_vis: u8,
    pub radius_vis_watchtower: u8,
    pub hp_structure: EnumMap<StructureKind, u8>,
    pub cost_structure: EnumMap<StructureKind, u16>,
    pub cost_action: EnumMap<ActionKind, u16>,
    pub cost_strike_tile: u16,
    pub dur_land_protect: Duration,
    pub dur_capture_city: Duration,
    pub dur_action_cooldown: EnumMap<ActionKind, Duration>,
    pub dur_action_delay: EnumMap<ActionKind, Duration>,
    pub dur_reveal_mineexplosion: Duration,
    pub dur_stun: Duration,
    pub dur_blind: Duration,
    pub dur_smoke: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct MapDescriptor {
    pub size: u8,
    pub topology: Topology,
}

/// The basic per-tile data, as per mapgen or file
#[bitfield]
#[derive(Clone, Copy, Default)]
pub struct MapGenTileData {
    pub kind: TileKind,
    pub mark: bool,
    pub item: ItemKind,
    #[skip] __: B2,
    pub region: u8,
}
