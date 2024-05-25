use mw_common::grid::*;
use mw_common::game::*;

use crate::prelude::*;
use crate::config::MapParams;

use std::sync::Weak;

#[derive(Clone)]
pub struct MapManager {
    inner: Arc<Mutex<MapManagerInner>>,
}

pub struct Map {
    data: MapDataPos<MapGenTileData>,
    cits: Vec<Pos>,
}

struct MapManagerInner {
    tx_requests: TxMpsc<(MapParams, TxOneshot<AnyResult<Arc<Map>>>)>,
    maps: HashMap<MapParams, Weak<Map>>,
}

impl MapManager {
    pub fn new() -> MapManager {
        let (tx_requests, rx_requests) = tokio::sync::mpsc::channel(1);
        tokio::spawn(map_service_task(rx_requests));
        let inner = MapManagerInner {
            tx_requests,
            maps: Default::default(),
        };
        MapManager { inner: Arc::new(Mutex::new(inner)) }
    }
    pub async fn clear_cache(&self) {
        self.inner.lock().await.maps.clear();
    }
    pub async fn load_map(&self, params: &MapParams) -> AnyResult<Arc<Map>> {
        let tx_requests = {
            let inner = self.inner.lock().await;
            if let Some(weak) = inner.maps.get(params) {
                if let Some(arc) = weak.upgrade() {
                    return Ok(arc);
                }
            }
            inner.tx_requests.clone()
        };

        let (tx_map, rx_map) = tokio::sync::oneshot::channel();
        tx_requests.send((params.clone(), tx_map)).await
            .context("Could not request map from map service")?;
        let arc = rx_map.await
            .context("Could not receive map from map service")??;

        {
            let mut inner = self.inner.lock().await;
            if let Some(weak) = inner.maps.get_mut(params) {
                *weak = Arc::downgrade(&arc);
            } else {
                inner.maps.insert(params.clone(), Arc::downgrade(&arc));
            }
        }

        Ok(arc)
    }
}

pub async fn map_service_task(
    mut rx_requests: RxMpsc<(MapParams, TxOneshot<AnyResult<Arc<Map>>>)>,
) {
    let mut q = Default::default();
    while let Some((params, tx_map)) = rx_requests.recv().await {
        let r_map = load_or_generate_map(&mut q, &params).await;
        tx_map.send(r_map);
    }
}

async fn load_or_generate_map(
    q: &mut mw_common::algo::FloodQ,
    params: &MapParams,
) -> AnyResult<Arc<Map>> {
    use crate::config::MapStyle;
    match params {
        MapParams::File { map_path } => {
            todo!();
        }
        MapParams::Generate { map_topology, map_style, map_seed, map_size, map_n_cits, map_land_bias } => {
            match map_style {
                MapStyle::Flat => {
                    todo!();
                }
                MapStyle::MineWars => {
                    #[cfg(feature = "proprietary")]
                    todo!();
                    // {
                    //     let arc = tokio::task::block_in_place(|| {
                    //         let (map, cits) = mw_proprietary_host::generate_minewars_map(
                    //             q,
                    //             *map_size,
                    //             *map_seed,
                    //             *map_land_bias,
                    //             *map_n_cits,
                    //         );
                    //         Arc::new(Map {
                    //             data: MapDataTopo::Hex(map),
                    //             cits,
                    //         })
                    //     });
                    //     Ok(arc)
                    // }
                    #[cfg(not(feature = "proprietary"))]
                    {
                        bail!("MineWars map generation not available in open-source builds!");
                    }
                }
            }
        }
    }
}
