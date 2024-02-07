use crate::prelude::*;

/// Asset type for a 3D MineWars Skin / Asset Pack definition
#[derive(Asset, Deserialize, Debug, Clone, TypePath)]
pub struct Ass3dConfig {
    pub gltf: PathBuf,
    pub name: String,
    pub tile_size: f32,
    pub water_hightide: Option<f32>,
    pub water_lowtide: Option<f32>,
    pub lod0: Ass3dLodConfig,
    pub lod1: Ass3dLodConfig,
    pub lod2: Ass3dLodConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Ass3dLodConfig {
    pub tiles_water: Option<TileOrTileSet>,
    pub tiles_regular: Option<TileOrTileSet>,
    pub tiles_fertile: Option<TileOrTileSet>,
    pub tiles_forest: Option<TileOrTileSet>,
    pub tiles_mountain: Option<TileOrTileSet>,
    pub tiles_destroyed: Option<TileOrTileSet>,
    pub tiles_harvested_fertile: Option<TileOrTileSet>,
    pub tiles_harvested_regular: Option<TileOrTileSet>,
    pub tiles_harvested_forest: Option<TileOrTileSet>,
    pub tiles_harvested_mountain: Option<TileOrTileSet>,
    pub tiles_road: Option<TileOrTileSet>,
    pub tile_cit: Option<Vec<String>>,
    pub tile_tower: Option<Vec<String>>,
    pub tile_wallmid: Option<Vec<String>>,
    pub wall_mtn_interface: Option<Vec<String>>,
    pub tileset_bridge: Option<Vec<Vec<String>>>,
    pub trees: Option<Vec<String>>,
    pub tree_radius_min: Option<f32>,
    pub tree_radius_max: Option<f32>,
    pub flag: Option<Vec<String>>,
    pub item_mine: Option<Vec<String>>,
    pub item_decoy: Option<Vec<String>>,
    pub item_trap: Option<Vec<String>>,
    pub digits: Option<Vec<String>>,
    pub digits_asterisk: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TileOrTileSet {
    Tile(Vec<String>),
    TileSet(Vec<Vec<String>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ass3dLod {
    Lod0,
    Lod1,
    Lod2,
}

impl Ass3dConfig {
    pub fn get_lod(&self, lod: Ass3dLod) -> &Ass3dLodConfig {
        match lod {
            Ass3dLod::Lod0 => &self.lod0,
            Ass3dLod::Lod1 => &self.lod1,
            Ass3dLod::Lod2 => &self.lod2,
        }
    }
}
