use mw_common::{prelude::*, game::ItemKind};

use super::MyRng;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MineGenSettings {
    /// Probability of an item appearing on a tile.
    pub mine_density: u8,
    /// Probability a mine being replaced by a decoy instead.
    pub prob_decoy: u8,
}

impl Default for MineGenSettings {
    fn default() -> Self {
        Self {
            mine_density: 96,
            prob_decoy: 64,
        }
    }
}

pub fn gen_mines<C: Coord, D, L: MapDataLayout<C>>(
    settings: &MineGenSettings,
    mapdata: &mut MapData<C, D, L>,
    rng: &mut MyRng,
    f_get_kind: impl Fn(&D) -> TileKind,
    f_set_item: impl Fn(&mut D, ItemKind),
) {
    for (_, d) in mapdata.iter_mut() {
        if f_get_kind(d).is_land() {
            if rng.gen_bool(settings.mine_density as f64 / 255.0) {
                if rng.gen_bool(settings.prob_decoy as f64 / 255.0) {
                    f_set_item(d, ItemKind::Decoy);
                } else {
                    f_set_item(d, ItemKind::Mine);
                }
            } else {
                f_set_item(d, ItemKind::Safe);
            }
        }
    }
}
