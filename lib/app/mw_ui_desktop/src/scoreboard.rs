use mw_app_core::{assets::SpritesAssets, player::{Plid, PlidColor, PlidScore, PlidState}, session::NeedsSessionGovernorSet};
use mw_ui_common::widgets::{multilayer_image::{ImageLayer, MultilayerImage}, WidgetsUiUpdateSS};

mod mini;
mod full;

use crate::{assets::sprite, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_plugins((
        self::mini::plugin,
        self::full::plugin,
    ));
    app.add_systems(Update, (
        update_plid_icons
            .run_if(any_with_component::<PlidIcon>)
            .in_set(NeedsSessionGovernorSet)
            .in_set(SetStage::Prepare(WidgetsUiUpdateSS)),
    ));
}

#[derive(Component)]
struct PlidIcon(PlayerId, Entity);

fn spawn_plid_icon(
    commands: &mut Commands,
    assets_spr: &SpritesAssets,
    e_plid: Entity,
    plid: PlayerId,
    size: f32,
) -> Entity {
    let e = commands.spawn((
        PlidIcon(plid, e_plid),
        MultilayerImage {
            layers: vec![
                ImageLayer {
                    image: UiImage {
                        color: Color::WHITE,
                        texture: assets_spr.ui_icons_img.clone(),
                        ..Default::default()
                    },
                    atlas: Some(TextureAtlas {
                        layout: assets_spr.ui_icons_layout.clone(),
                        index: sprite::TILE_SOLID,
                    }),
                    force_height: Some(size),
                    force_width: None,
                },
                ImageLayer {
                    image: UiImage {
                        color: Color::WHITE,
                        texture: assets_spr.numbers_img.clone(),
                        ..Default::default()
                    },
                    atlas: Some(TextureAtlas {
                        layout: assets_spr.numbers_layout.clone(),
                        index: 0,
                    }),
                    force_height: Some(size),
                    force_width: None,
                },
            ],
            background_color: Color::NONE.into(),
        },
    )).id();
    e
}

fn update_plid_icons(
    assets_spr: Res<SpritesAssets>,
    q_player: Query<(
        &PlidColor,
        &PlidScore,
        &PlidState,
    ), With<Plid>>,
    mut q_icon: Query<(&PlidIcon, &mut MultilayerImage)>,
) {
    // FIXME: PERF: this changes values over and over every frame;
    // figure out a RC or something so it doesn't run every frame.

    for (icon, mut multilayer) in &mut q_icon {
        let (color, score, state) = q_player.get(icon.1).unwrap();
        match *state {
            PlidState::Alive => {
                multilayer.layers[0].image.color = color.color;
                multilayer.layers[1].image.texture = assets_spr.numbers_img.clone();
                if let Some(atlas) = &mut multilayer.layers[1].atlas {
                    atlas.layout = assets_spr.numbers_layout.clone();
                    atlas.index = score.0 as usize;
                }
            },
            PlidState::Dead { .. } => {
                multilayer.layers[0].image.color = color.color.with_alpha(0.5);
                multilayer.layers[1].image.color = Color::WHITE.with_alpha(0.5);
                multilayer.layers[1].image.texture = assets_spr.numbers_img.clone();
                if let Some(atlas) = &mut multilayer.layers[1].atlas {
                    atlas.layout = assets_spr.numbers_layout.clone();
                    atlas.index = score.0 as usize;
                }
            },
            PlidState::Eliminated => {
                multilayer.layers[0].image.color = color.color.with_alpha(0.5);
                multilayer.layers[1].image.color = Color::WHITE.with_alpha(0.5);
                multilayer.layers[1].image.texture = assets_spr.ui_icons_img.clone();
                if let Some(atlas) = &mut multilayer.layers[1].atlas {
                    atlas.layout = assets_spr.ui_icons_layout.clone();
                    atlas.index = sprite::ICON_X;
                }
            },
        }
    }
}
