use bevy::render::{render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureDimension, TextureFormat}};
use mw_common::{game::{MapDescriptor, TileKind}, grid::Topology};

use crate::{prelude::*, settings::SettingsSyncSS};
use crate::map::{MwTilePos, TileOwner, NeedsMapSet, MapUpdateSet, TileAlert};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_minimap);
    app.add_systems(Update, (
        update_minimap
            .in_set(NeedsMapSet)
            .in_set(SetStage::Want(SettingsSyncSS))
            .after(MapUpdateSet::TileOwner) // PERF ?
            .run_if(resource_exists::<MinimapImage>),
    ));
}

#[derive(Resource)]
pub struct MinimapImage(pub Handle<Image>);

fn setup_minimap(
    mut commands: Commands,
    mut ass_image: ResMut<Assets<Image>>,
) {
    let image = Image::new_fill(
        Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        }, TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    );
    let handle = ass_image.add(image);
    commands.insert_resource(MinimapImage(handle));
}

fn update_minimap(
    settings: Res<AllSettings>,
    minimap_handle: Res<MinimapImage>,
    mut ass_image: ResMut<Assets<Image>>,
    desc: Res<MapDescriptor>,
    q_tile: Query<
        (&MwTilePos, Option<&TileOwner>, &TileKind, Option<&TileAlert>),
        Or<(Changed<TileOwner>, Changed<TileKind>, With<TileAlert>)>
    >,
) {
    if desc.is_changed() || settings.is_changed() {
        let minimap_image = ass_image.get_mut(&minimap_handle.0)
            .expect("minimap image must exist");

        let span = desc.size as u32 * 2 + 1;
        let (height, width) = match desc.topology {
            Topology::Hex => {
                let tile_size = settings.ui_hud.minimap_scale.max(2) as u32;
                let y_extra = tile_size / 2;
                (
                   span * (tile_size + y_extra * 2) + tile_size + 1,
                   span * tile_size * 2,
                )
            },
            Topology::Sq => {
                let tile_size = settings.ui_hud.minimap_scale.max(2) as u32 - 1;
                (
                   span * (tile_size * 2 + 1),
                   span * (tile_size * 2 + 1),
                )
            },
        };
        *minimap_image = Image::new_fill(
            Extent3d {
                width, height, depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        );
    }

    for (pos, owner, kind, alert) in &q_tile {
        let minimap_image = ass_image.get_mut(&minimap_handle.0)
            .expect("minimap image must exist");
        let w = minimap_image.texture_descriptor.size.width as i32;
        let h = minimap_image.texture_descriptor.size.height as i32;

        let mut rgba = owner
            .map(|o| Color::from(settings.player_colors.visible[o.0.i()]).as_rgba_u8())
            .unwrap_or([0, 0, 0, 0]);

        if let Some(alert) = alert {
            // FIXME: this can get stuck on white
            // if the last frame before the TileAlert component was removed
            // triggers this
            if alert.0.elapsed_secs().rem_euclid(0.26) < 0.125 {
                rgba = [255, 255, 255, 255];
            }
        }
        match desc.topology {
            Topology::Hex => {
                let tile_size = settings.ui_hud.minimap_scale.max(2) as i32;
                let y_extra = tile_size / 2;
                let cy = pos.0.0 as i32;
                let cx = pos.0.1 as i32;
                let y = h / 2 - cy * (tile_size + y_extra * 2);
                let x = w / 2 + cx * (tile_size * 2) + cy * tile_size;
                for yy in (y-(tile_size+y_extra))..=(y+(tile_size+y_extra)) {
                    let t = ((yy - y).abs()-y_extra).max(0);
                    for xx in (x-(tile_size-t))..(x+(tile_size-t)) {
                        if yy < 0 || xx < 0 || yy >= h || xx >= w {
                            continue;
                        }
                        let i = (yy * w + xx) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                    }
                }
                match kind {
                    TileKind::Mountain => {
                        let rgba = [0, 0, 0, 255];
                        let i = (y * w + x) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                        let i = (y * w + x - 1) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                        let i = (y * w + x + 1) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                        let i = ((y - 1) * w + x) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                    }
                    TileKind::Forest => {
                        let rgba = [0, 0, 0, 255];
                        let i = ((y - 1) * w + x - 1) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                        let i = ((y + 1) * w + x + 1) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                    }
                    _ => {}
                }
            }
            Topology::Sq => {
                let tile_size = settings.ui_hud.minimap_scale.max(2) as i32 - 1;
                let y = h / 2 - pos.0.0 as i32 * (tile_size * 2 + 1);
                let x = w / 2 + pos.0.1 as i32 * (tile_size * 2 + 1);
                for yy in (y-tile_size)..=(y+tile_size) {
                    for xx in (x-tile_size)..=(x+tile_size) {
                        if yy < 0 || xx < 0 || yy >= h || xx >= w {
                            continue;
                        }
                        let i = (yy * w + xx) as usize * 4;
                        minimap_image.data[i..(i+4)].copy_from_slice(&rgba);
                    }
                }
            }
        }
    }
}
