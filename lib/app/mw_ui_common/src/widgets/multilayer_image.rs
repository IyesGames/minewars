use crate::prelude::*;

use super::WidgetsUiUpdateSS;

pub fn plugin(app: &mut App) {
    app.configure_sets(
        Update, MultilayerImageSet
            .run_if(any_with_component::<MultilayerImage>)
    );
    app.add_systems(Update, (
        update_multilayer_images
            .in_set(SetStage::Provide(WidgetsUiUpdateSS))
            .in_set(MultilayerImageSet),
    ));
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MultilayerImageSet;

#[derive(Component)]
pub struct MultilayerImage {
    pub layers: Vec<ImageLayer>,
    pub background_color: BackgroundColor,
}

pub struct ImageLayer {
    pub image: UiImage,
    pub atlas: Option<TextureAtlas>,
    pub force_width: Option<f32>,
    pub force_height: Option<f32>,
}

#[derive(Component)]
struct MultilayerImageMember {
    root: Entity,
    layer: usize,
}

#[derive(Component)]
struct MultilayerImageMembers {
    members: Vec<Entity>,
}

fn update_multilayer_images(
    mut commands: Commands,
    mut q_member: Query<(&mut Style, &mut UiImage, Option<&mut TextureAtlas>), With<MultilayerImageMember>>,
    mut q_root: Query<(Entity, Ref<MultilayerImage>, Option<&mut MultilayerImageMembers>, Option<&mut BackgroundColor>)>,
) {
    let member_style = Style {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..Default::default()
    };
    for (e_root, multilayer, members, background) in &mut q_root {
        if let Some(mut members) = members {
            // already set up; check for changes
            if !multilayer.is_changed() {
                continue;
            }
            let mut background = background.unwrap();
            if *background != multilayer.background_color {
                *background = multilayer.background_color;
            }
            let n_layers = multilayer.layers.len();
            let n_members = members.members.len();
            // despawn any extra old members
            if n_layers < n_members {
                let e_final = members.members[n_layers - 1];
                commands.entity(e_final).despawn_descendants();
            }
            // spawn any extra new members
            let mut last_parent = *members.members.last().unwrap();
            for (i, layer) in multilayer.layers.iter().enumerate().skip(n_members) {
                let mut style = member_style.clone();
                if let Some(width) = layer.force_width {
                    style.width = Val::Px(width);
                }
                if let Some(height) = layer.force_height {
                    style.height = Val::Px(height);
                }
                let e = commands.spawn((
                    ImageBundle {
                        style,
                        image: layer.image.clone(),
                        ..Default::default()
                    },
                    MultilayerImageMember {
                        root: e_root,
                        layer: i,
                    },
                )).id();
                if let Some(atlas) = &layer.atlas {
                    commands.entity(e).insert(atlas.clone());
                }
                members.members.push(e);
                commands.entity(last_parent).add_child(e);
                last_parent = e;
            }
            // update anything that is changed within existing members
            for (layer, e_member) in multilayer.layers.iter().zip(members.members.iter()) {
                let (mut style, mut image, atlas) = q_member.get_mut(*e_member).unwrap();
                // FIXME: UiImage is missing PartialEq
                // if layer.image != *image {
                    *image = layer.image.clone();
                // }
                style.width = layer.force_width.map(|x| Val::Px(x))
                    .unwrap_or(Val::Auto);
                style.height = layer.force_height.map(|x| Val::Px(x))
                    .unwrap_or(Val::Auto);
                match (atlas, &layer.atlas) {
                    (None, None) => {},
                    (None, Some(new)) => {
                        commands.entity(*e_member).insert(new.clone());
                    },
                    (Some(_), None) => {
                        commands.entity(*e_member).remove::<TextureAtlas>();
                    },
                    (Some(mut old), Some(new)) => {
                        // FIXME: TextureAtlas is missing PartialEq
                        // if *old != *new {
                            *old = new.clone();
                        // }
                    },
                }
            }
        } else {
            // first-time setup
            let mut members = MultilayerImageMembers {
                members: vec![e_root],
            };
            let mut iter_layers = multilayer.layers.iter().enumerate();
            if let Some((i, first)) = iter_layers.next() {
                let mut style = member_style.clone();
                if let Some(width) = first.force_width {
                    style.width = Val::Px(width);
                }
                if let Some(height) = first.force_height {
                    style.height = Val::Px(height);
                }
                commands.entity(e_root).insert((
                    ImageBundle {
                        background_color: multilayer.background_color.clone(),
                        style,
                        image: first.image.clone(),
                        ..Default::default()
                    },
                    MultilayerImageMember {
                        root: e_root,
                        layer: i,
                    },
                ));
                if let Some(atlas) = &first.atlas {
                    commands.entity(e_root).insert(atlas.clone());
                }
            }
            let mut last_parent = e_root;
            for (i, layer) in iter_layers {
                let mut style = member_style.clone();
                if let Some(width) = layer.force_width {
                    style.width = Val::Px(width);
                }
                if let Some(height) = layer.force_height {
                    style.height = Val::Px(height);
                }
                let e = commands.spawn((
                    ImageBundle {
                        style,
                        image: layer.image.clone(),
                        ..Default::default()
                    },
                    MultilayerImageMember {
                        root: e_root,
                        layer: i,
                    },
                )).id();
                if let Some(atlas) = &layer.atlas {
                    commands.entity(e).insert(atlas.clone());
                }
                members.members.push(e);
                commands.entity(last_parent).add_child(e);
                last_parent = e;
            }
            commands.entity(e_root).insert(members);
        }
    }
}
