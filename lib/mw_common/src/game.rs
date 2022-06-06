//! Stuff related to the MineWars gameplay
//!
//! Types to represent game data, functions to process it, …

use enum_map::{Enum, EnumMap};
use crate::grid::map::{CompactMapCoordExt, MapData};
use crate::{algo, HashMap};

pub type Res = u32;
pub type CitId = u8;

/// Different variants of mines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum)]
pub enum MineKind {
    Mine,
    Decoy,
}

/// Is something in-production or completed?
///
/// (for road tiles, or maybe other future buildables)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProdState {
    Pending,
    Built,
}

/// All the various things that a city can be working on producing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum)]
pub enum ProdItem {
    Mine(MineKind),
    Road,
}

/// Items that a player can have in their inventory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum)]
pub enum InventoryItem {
    Mine(MineKind),
}

/// The base variant of a map tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum)]
pub enum TileKind {
    Water,
    Regular,
    Fertile,
    Mountain,
    Road,
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
            TileKind::Mountain => false,
            TileKind::Regular |
            TileKind::Fertile |
            TileKind::Road => true,
        }
    }
}

/// The game rules / balancing table
pub struct GameParams {
    pub res_base: Res,
    pub res_local: EnumMap<TileKind, Res>,
    pub res_export: EnumMap<TileKind, Res>,
    pub radius_vis: u8,
    pub radius_fertile: u8,
    pub costs: EnumMap<ProdItem, Res>,
}

/// The per-tile data used for initializing a map
///
/// This is what the world generation algo outputs.
/// This is what the replay file loader outputs.
/// This is what the network protocol init stage outputs.
///
/// This is to be converted into a more complete "tile state"
/// struct on the client or server side, as appropriate.
#[derive(Debug, Clone)]
pub struct MapTileInit {
    pub kind: TileKind,
    pub mine: Option<MineKind>,
    pub region: CitId,
    pub cit: bool,
    /// Used as a temporary flag by algorithms, do not preserve
    pub mark: bool,
}

pub struct MapDataInit<C: CompactMapCoordExt> {
    pub map: MapData<C, MapTileInit>,
    pub cits: Vec<C>,
    pub mines: HashMap<C, MineKind>,
}
