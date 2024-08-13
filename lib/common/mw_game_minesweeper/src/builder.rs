use super::*;

pub struct GameMinesweeperBuilder {
    settings: MinesweeperSettings,
    playerdata: Vec<PlayerData>,
}

impl GameMinesweeperBuilder {
    pub fn new(settings: MinesweeperSettings, starting_plids: u8) -> Self {
        Self {
            playerdata: vec![PlayerData {
                n_owned: 0,
                n_lives: settings.n_lives,
            }; starting_plids as usize],
            settings,
        }
    }
    pub fn with_mapdata_hex(
        self,
        map_size: u8,
        f_kind: impl Fn(Hex) -> TileKind,
    ) -> GameMinesweeper {
        GameMinesweeper::Hex(
            self.with_mapdata_inner(map_size, f_kind)
        )
    }
    pub fn with_mapdata_sq(
        self,
        map_size: u8,
        f_kind: impl Fn(Sq) -> TileKind,
    ) -> GameMinesweeper {
        GameMinesweeper::Sq(
            self.with_mapdata_inner(map_size, f_kind)
        )
    }
    fn with_mapdata_inner<C: Coord>(
        self,
        map_size: u8,
        f_kind: impl Fn(C) -> TileKind,
    ) -> GameMinesweeperTopo<C> {
        let mut default_tile = TileData::default();
        default_tile.set_kind(TileKind::Regular);
        default_tile.set_owner(0);
        default_tile.set_flag(0);
        let mut mapdata = MapDataC::<C, TileData>::new(map_size, default_tile);
        let mut n_unexplored_tiles = 0;
        mapdata.iter_mut().for_each(|(c, d)| {
            let kind = f_kind(c);
            if kind.is_land() {
                n_unexplored_tiles += 1;
            }
            d.set_kind(kind);
        });
        GameMinesweeperTopo {
            settings: self.settings,
            mapdata,
            playerdata: self.playerdata,
            n_unexplored_tiles,
            floodq: Default::default(),
            rng: MyRng::from_entropy(),
        }
    }
}
