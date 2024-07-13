use bevy::asset::RecursiveDependencyLoadState;
use bevy::gltf::Gltf;

use crate::prelude::*;
use crate::assets::ass3d::*;

use super::map::{Ass3dTileKind, TileAss3d};

pub fn plugin(app: &mut App) {
    app.init_resource::<Ass3dResolver>();
    app.add_systems(OnExit(AppState::StartupLoading), init_ass3d_resolver);
}

#[derive(Debug)]
pub struct ResolvedAsset {
    pub scale: f32,
    pub scene: Handle<Scene>,
}

#[derive(Debug)]
enum Ass3dGltfState {
    NotLoaded,
    Loading(Handle<Gltf>),
    Loaded(Handle<Gltf>),
    Errored,
}

fn init_ass3d_resolver(
    mut resolver: ResMut<Ass3dResolver>,
    ass_game: Res<crate::assets::Gfx3dAssets>,
    ass_ass3d: Res<Assets<Ass3dConfig>>,
    ass_server: Res<AssetServer>,
) { // TODO: Progress-ify
    resolver.push_skin(ass_game.fallback_3d.clone());
    // FIXME
    resolver.load_gltfs(&*ass_ass3d, &*ass_server);
}

#[derive(Resource, Default)]
pub struct Ass3dResolver {
    skin_priolist: Vec<(Handle<Ass3dConfig>, Ass3dGltfState)>,
}

impl Ass3dResolver {
    pub fn push_skin(&mut self, skin: Handle<Ass3dConfig>) {
        self.skin_priolist.insert(0, (skin, Ass3dGltfState::NotLoaded));
    }
    pub fn pop_skin(&mut self) {
        self.skin_priolist.remove(0);
    }

    pub fn load_gltfs(
        &mut self,
        ass_ass3d: &Assets<Ass3dConfig>,
        ass_server: &AssetServer,
    ) -> Progress {
        let mut progress = Progress {
            done: 0,
            total: self.skin_priolist.len() as u32,
        };
        for (skin_handle, gltf_state) in &mut self.skin_priolist {
            let Some(gltf_path) = ass_ass3d.get(&*skin_handle).map(|skin| &skin.gltf) else {
                continue;
            };
            loop {
                match gltf_state {
                    Ass3dGltfState::NotLoaded => {
                        let skin_path = skin_handle.path()
                            .expect("Skin Ass3d assets must be real files!");
                        let parent_dir = skin_path.path().parent().unwrap_or(Path::new(""));
                        let gltf_fullpath = parent_dir.join(gltf_path);
                        let gltf_handle = ass_server.load(gltf_fullpath);
                        *gltf_state = Ass3dGltfState::Loading(gltf_handle);
                    }
                    Ass3dGltfState::Loading(gltf_handle) => {
                        match ass_server.recursive_dependency_load_state(&*gltf_handle) {
                            RecursiveDependencyLoadState::Loading => {
                                break;
                            },
                            RecursiveDependencyLoadState::Loaded => {
                                *gltf_state = Ass3dGltfState::Loaded(gltf_handle.clone());
                            }
                            RecursiveDependencyLoadState::Failed => {
                                *gltf_state = Ass3dGltfState::Errored;
                            }
                            RecursiveDependencyLoadState::NotLoaded => unreachable!(),
                        }
                    }
                    Ass3dGltfState::Loaded(_) | Ass3dGltfState::Errored => {
                        progress.done += 1;
                        break;
                    }
                }
            }
        }
        progress
    }

    pub fn get_tile_asset(
        &self,
        ass_ass3d: &Assets<Ass3dConfig>,
        ass_gltf: &Assets<Gltf>,
        lod: Ass3dLod,
        ass3d: &TileAss3d,
    ) -> Option<ResolvedAsset> {
        for (skin, gltf) in self.iter_available_skins(ass_ass3d, ass_gltf) {
            let ass_lod = skin.get_lod(lod);
            let scene_name = match ass3d.kind {
                Ass3dTileKind::BadTile => todo!(),
                Ass3dTileKind::Water =>
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_water, lod, ass3d),
                Ass3dTileKind::Regular =>
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_regular, lod, ass3d),
                Ass3dTileKind::Fertile => 
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_fertile, lod, ass3d),
                Ass3dTileKind::Forest => 
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_forest, lod, ass3d),
                Ass3dTileKind::Mountain => 
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_mountain, lod, ass3d),
                Ass3dTileKind::Destroyed => 
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_destroyed, lod, ass3d),
                Ass3dTileKind::HarvestedFertile =>
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_harvested_fertile, lod, ass3d),
                Ass3dTileKind::HarvestedRegular =>
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_harvested_regular, lod, ass3d),
                Ass3dTileKind::HarvestedForest =>
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_harvested_forest, lod, ass3d),
                Ass3dTileKind::HarvestedMountain =>
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_harvested_mountain, lod, ass3d),
                Ass3dTileKind::Road =>
                    Self::resolve_tile_or_tileset(&ass_lod.tiles_road, lod, ass3d),
                Ass3dTileKind::Cit =>
                    Self::resolve_tile(ass_lod.tile_cit.as_ref(), lod, ass3d),
                Ass3dTileKind::Tower =>
                    Self::resolve_tile(ass_lod.tile_tower.as_ref(), lod, ass3d),
                Ass3dTileKind::WallMid =>
                    Self::resolve_tile(ass_lod.tile_wallmid.as_ref(), lod, ass3d),
                Ass3dTileKind::Bridge =>
                    Self::resolve_tileset(ass_lod.tileset_bridge.as_ref(), lod, ass3d),
            };
            if !scene_name.is_empty() {
                if let Some(scene) = gltf.named_scenes.get(scene_name) {
                    return Some(ResolvedAsset {
                        scale: crate::misc::TILE_SCALE / skin.tile_size,
                        scene: scene.clone(),
                    })
                }
            }
        }
        None
    }

    fn resolve_tile_or_tileset<'t>(tile_or_tileset: &'t Option<TileOrTileSet>, lod: Ass3dLod, ass3d: &TileAss3d) -> &'t str {
        match tile_or_tileset {
            Some(TileOrTileSet::TileSet(config)) => {
                Self::resolve_tileset(Some(config), lod, ass3d)
            }
            Some(TileOrTileSet::Tile(config)) => {
                Self::resolve_tile(Some(config), lod, ass3d)
            }
            None => "",
        }
    }
    fn resolve_tileset<'t>(config: Option<&'t Vec<Vec<String>>>, lod: Ass3dLod, ass3d: &TileAss3d) -> &'t str {
        if let Some(config) = config {
            let variant_config = &config[ass3d.variant as usize];
            if variant_config.len() == 0 {
                return "";
            }
            let subvariant = ass3d.subvariant[lod as usize] as usize % variant_config.len();
            variant_config[subvariant].as_str()
        } else {
            ""
        }
    }
    fn resolve_tile<'t>(config: Option<&'t Vec<String>>, lod: Ass3dLod, ass3d: &TileAss3d) -> &'t str {
        if let Some(config) = config {
            config[ass3d.subvariant[lod as usize] as usize % config.len()].as_str()
        } else {
            ""
        }
    }

    pub fn get_digit_asset(
        &self,
        ass_ass3d: &Assets<Ass3dConfig>,
        ass_gltf: &Assets<Gltf>,
        lod: Ass3dLod,
        digit: u8,
        asterisk: bool,
    ) -> Option<ResolvedAsset> {
        for (skin, gltf) in self.iter_available_skins(ass_ass3d, ass_gltf) {
            let ass_lod = skin.get_lod(lod);
            let digits = if asterisk {
                ass_lod.digits_asterisk.as_ref()
            } else {
                ass_lod.digits.as_ref()
            };
            if let Some(digits) = digits {
                if let Some(digit_scene_name) = digits.get(digit as usize) {
                    if let Some(scene) = gltf.named_scenes.get(digit_scene_name.as_str()) {
                        return Some(ResolvedAsset {
                            scale: crate::misc::TILE_SCALE / skin.tile_size,
                            scene: scene.clone(),
                        });
                    }
                }
            }
        }
        None
    }

    fn iter_available_skins<'a>(
        &'a self,
        ass_ass3d: &'a Assets<Ass3dConfig>,
        ass_gltf: &'a Assets<Gltf>,
    ) -> impl Iterator<Item = (&'a Ass3dConfig, &'a Gltf)> + 'a {
        self.skin_priolist.iter()
            .filter_map(|(handle_ass3d, gltf_state)| {
                match (ass_ass3d.get(handle_ass3d), gltf_state) {
                    (Some(ass3d), Ass3dGltfState::Loaded(handle_gltf)) => {
                        ass_gltf.get(handle_gltf).and_then(|gltf| Some((ass3d, gltf)))
                    }
                    _ => None,
                }
            })
    }
}
