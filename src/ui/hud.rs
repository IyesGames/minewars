use bevy::window::PrimaryWindow;

use crate::{prelude::*, assets::UiAssets, ui, minimap::MinimapImage};

use super::{tooltip::InfoAreaText, UiRoot};

pub(super) struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_hud);
        app.add_systems(Update, minimap_image_scale_fixme.run_if(resource_exists::<MinimapImage>()));
    }
}

pub fn spawn_cityentry_unowned(
    commands: &mut Commands,
    uiassets: &UiAssets,
    citid: u8,
) -> Entity {
    let wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.5, 0.5, 0.0)),
            ..Default::default()
        },
    )).id();
    let icon = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.3, 0.3, 0.0)),
            ..Default::default()
        },
    )).id();
    let name = commands.spawn((
        // L10nKey(String::new()),
        TextBundle {
            style: Style {
                margin: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            text: Text::from_section(
                format!("City {}", citid),
                TextStyle {
                    font: uiassets.font_bold.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        },
    )).id();
    commands.entity(wrapper).push_children(&[icon, name]);
    wrapper
}

pub fn spawn_cityentry_owned(
    commands: &mut Commands,
    uiassets: &UiAssets,
    citid: u8,
) -> Entity {
    let wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.5, 0.5, 0.0)),
            ..Default::default()
        },
    )).id();
    let heading = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.6, 0.5, 0.0)),
            ..Default::default()
        },
    )).id();
    let icon = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.3, 0.3, 0.0)),
            ..Default::default()
        },
    )).id();
    let name = commands.spawn((
        // L10nKey(String::new()),
        TextBundle {
            style: Style {
                margin: UiRect {
                    left: Val::Px(4.0),
                    right: Val::Px(8.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::from_section(
                format!("City {}", citid),
                TextStyle {
                    font: uiassets.font_bold.clone(),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        },
    )).id();

    commands.entity(wrapper).push_children(&[heading]);
    commands.entity(heading).push_children(&[icon, name]);

    fn spawn_stat(commands: &mut Commands, uiassets: &UiAssets, label: &str, value: &str, value2: &str) -> Entity {
        let text_style_label = TextStyle {
            font: uiassets.font.clone(),
            font_size: 16.0,
            color: Color::WHITE,
        };
        let text_style_value = TextStyle {
            font: uiassets.font_light.clone(),
            font_size: 16.0,
            color: Color::WHITE,
        };
        let stats_area = commands.spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect {
                        left: Val::Px(4.0),
                        right: Val::Px(4.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                // background_color: BackgroundColor(Color::rgb(0.5, 0.6, 0.0)),
                ..Default::default()
            },
        )).id();
        let text_label = commands.spawn((
            TextBundle {
                text: Text::from_section(label, text_style_label.clone()),
                ..Default::default()
            },
        )).id();
        let text_value = commands.spawn((
            TextBundle {
                text: Text::from_section(value, text_style_value.clone()),
                ..Default::default()
            },
        )).id();
        let text_value2 = commands.spawn((
            TextBundle {
                text: Text::from_section(value2, text_style_value.clone()),
                ..Default::default()
            },
        )).id();
        commands.entity(stats_area).push_children(&[text_label, text_value, text_value2]);
        stats_area
    }
    let stats_area = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.5, 0.6, 0.0)),
            ..Default::default()
        },
    )).id();
    let stat_res = spawn_stat(commands, uiassets, "Res:", "420/666", "+ 1234");
    let stat_exp = spawn_stat(commands, uiassets, "Export:", "123/234", "x3");
    let stat_land = spawn_stat(commands, uiassets, "Land:", "600/1024", "(59.9%)");
    let prod_area = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.5, 0.6, 0.0)),
            ..Default::default()
        },
    )).id();
    let prod_icon = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(32.0),
                height: Val::Px(32.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.3, 0.3, 0.0)),
            ..Default::default()
        },
    )).id();
    let prod_timer = commands.spawn((
        TextBundle {
            text: Text::from_section(
                "59s",
                TextStyle {
                    font: uiassets.font_bold.clone(),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        },
    )).id();
    // commands.entity(col1).push_children(&[stat_res, stat_land]);
    // commands.entity(col2).push_children(&[stat_exp, stat_roads]);
    commands.entity(stats_area).push_children(&[stat_res, stat_exp, stat_land]);
    commands.entity(prod_area).push_children(&[prod_icon, prod_timer]);
    commands.entity(wrapper).push_children(&[stats_area, prod_area]);
    wrapper
}

fn spawn_playericon(commands: &mut Commands, uiassets: &UiAssets, color: Color, n: u32) -> Entity {
    let player = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                ..Default::default()
            },
            background_color: BackgroundColor(color),
            ..Default::default()
        },
    )).id();
    let text = commands.spawn((
        TextBundle {
            text: Text::from_section(
                format!("{}", n),
                TextStyle {
                    font: uiassets.font_bold.clone(),
                    font_size: 48.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        },
    )).id();
    commands.entity(player).push_children(&[text]);
    player
}

fn setup_hud(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
    minimap_image: Res<MinimapImage>,
) {
    let root = commands.spawn((
        StateDespawnMarker,
        UiRoot,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let topcenter = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Auto,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                ..Default::default()
            },
            z_index: ZIndex::Global(1),
            ..Default::default()
        },
    )).id();
    let bottom = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Auto,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::End,
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            z_index: ZIndex::Global(4),
            ..Default::default()
        },
    )).id();
    // let citylist = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             position_type: PositionType::Absolute,
    //             bottom: Val::Auto,
    //             left: Val::Px(0.0),
    //             right: Val::Auto,
    //             top: Val::Px(0.0),
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Stretch,
    //             justify_content: JustifyContent::Start,
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.3, 0.2, 0.0)),
    //         z_index: ZIndex::Global(2),
    //         ..Default::default()
    //     },
    // )).id();
    // let citylist_owned = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Stretch,
    //             justify_content: JustifyContent::Start,
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.2, 0.4, 0.0)),
    //         ..Default::default()
    //     },
    // )).id();
    // let citylist_unowned = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Row,
    //             align_items: AlignItems::Start,
    //             justify_content: JustifyContent::Start,
    //             flex_wrap: FlexWrap::Wrap,
    //             align_content: AlignContent::Start,
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.2, 0.5, 0.0)),
    //         ..Default::default()
    //     },
    // )).id();
    let playerbar = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.2, 0.3, 0.0)),
            ..Default::default()
        },
    )).id();
    let player1 = spawn_playericon(&mut commands, &uiassets, Color::rgb(0.5, 0.5, 0.0), 4);
    let player2 = spawn_playericon(&mut commands, &uiassets, Color::rgb(0.5, 0.5, 0.1), 2);
    let player3 = spawn_playericon(&mut commands, &uiassets, Color::rgb(0.6, 0.5, 0.0), 1);
    let player4 = spawn_playericon(&mut commands, &uiassets, Color::rgb(0.5, 0.6, 0.0), 1);
    let player5 = spawn_playericon(&mut commands, &uiassets, Color::rgb(0.4, 0.5, 0.0), 1);
    let player6 = spawn_playericon(&mut commands, &uiassets, Color::rgb(0.5, 0.4, 0.0), 1);
    let notify_area = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Auto,
                right: Val::Px(16.0),
                left: Val::Auto,
                top: Val::Px(96.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.3, 0.3, 0.3)),
            z_index: ZIndex::Global(3),
            ..Default::default()
        },
    )).id();
    let notify_text = commands.spawn((
        // L10nKey(String::new()),
        TextBundle {
            text: Text::from_section(
                "This is a notification from the game, yo!",
                TextStyle {
                    font: uiassets.font2.clone(),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        },
    )).id();
    let minimap = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.0, 0.0, 0.0)),
            ..Default::default()
        },
    )).id();
    let minimap_image = commands.spawn((
        ImageBundle {
            image: UiImage {
                texture: minimap_image.0.clone(),
                ..Default::default()
            },
            ..Default::default()
        },
        MinimapImageNode,
    )).id();
    // let inventory = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Center,
    //             justify_content: JustifyContent::SpaceBetween,
    //             width: Val::Px(200.0),
    //             height: Val::Px(140.0),
    //             padding: UiRect::all(Val::Px(4.0)),
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.0, 0.3, 0.3)),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_text = commands.spawn((
    //     // L10nKey(String::new()),
    //     TextBundle {
    //         text: Text::from_section(
    //             "INVENTORY",
    //             TextStyle {
    //                 font: uiassets.font_bold.clone(),
    //                 font_size: 32.0,
    //                 color: Color::WHITE,
    //             },
    //         ),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_contents = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Row,
    //             align_items: AlignItems::Center,
    //             justify_content: JustifyContent::SpaceEvenly,
    //             width: Val::Percent(100.0),
    //             height: Val::Auto,
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.0, 0.4, 0.3)),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_mines = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Center,
    //             justify_content: JustifyContent::Center,
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.0, 0.5, 0.3)),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_mines_icon = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             align_items: AlignItems::Center,
    //             justify_content: JustifyContent::Center,
    //             width: Val::Px(64.0),
    //             height: Val::Px(64.0),
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.5, 0.3, 0.0)),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_mines_text = commands.spawn((
    //     TextBundle {
    //         text: Text::from_section(
    //             "999",
    //             TextStyle {
    //                 font: uiassets.font.clone(),
    //                 font_size: 32.0,
    //                 color: Color::WHITE,
    //             },
    //         ),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_decoys = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Center,
    //             justify_content: JustifyContent::Center,
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.0, 0.3, 0.5)),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_decoys_icon = commands.spawn((
    //     NodeBundle {
    //         style: Style {
    //             align_items: AlignItems::Center,
    //             justify_content: JustifyContent::Center,
    //             width: Val::Px(64.0),
    //             height: Val::Px(64.0),
    //             ..Default::default()
    //         },
    //         background_color: BackgroundColor(Color::rgb(0.5, 0.3, 0.0)),
    //         ..Default::default()
    //     },
    // )).id();
    // let inventory_decoys_text = commands.spawn((
    //     TextBundle {
    //         text: Text::from_section(
    //             "999",
    //             TextStyle {
    //                 font: uiassets.font.clone(),
    //                 font_size: 32.0,
    //                 color: Color::WHITE,
    //             },
    //         ),
    //         ..Default::default()
    //     },
    // )).id();
    let bot_midarea = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                flex_grow: 1.0,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    let info_area = commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Auto,
                padding: UiRect {
                    top: Val::Px(4.0),
                    bottom: Val::Px(4.0),
                    left: Val::Px(4.0),
                    right: Val::Auto,
                },
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.0, 0.0, 0.0)),
            ..Default::default()
        },
    )).id();
    let info_text = commands.spawn((
        InfoAreaText,
        // L10nKey(String::new()),
        TextBundle {
            text: Text::from_section(
                "Info area text!",
                TextStyle {
                    font: uiassets.font2_light.clone(),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ),
            ..Default::default()
        },
    )).id();
    let toolbar = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    let tool1 = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.3, 0.0, 0.3)),
            ..Default::default()
        },
    )).id();
    let tool2 = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.4, 0.0, 0.3)),
            ..Default::default()
        },
    )).id();
    let tool3 = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.3, 0.0, 0.4)),
            ..Default::default()
        },
    )).id();
    let tool4 = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(64.0),
                height: Val::Px(64.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::rgb(0.3, 0.1, 0.3)),
            ..Default::default()
        },
    )).id();

    // let city1 = spawn_cityentry_unowned(&mut commands, &uiassets, 1);
    // let city2 = spawn_cityentry_owned(&mut commands, &uiassets, 2);
    // let city3 = spawn_cityentry_owned(&mut commands, &uiassets, 3);
    // let city4 = spawn_cityentry_unowned(&mut commands, &uiassets, 4);
    // let city5 = spawn_cityentry_owned(&mut commands, &uiassets, 5);
    // let city6 = spawn_cityentry_owned(&mut commands, &uiassets, 6);
    // let city7 = spawn_cityentry_unowned(&mut commands, &uiassets, 7);
    // let city8 = spawn_cityentry_unowned(&mut commands, &uiassets, 8);
    // let city9 = spawn_cityentry_unowned(&mut commands, &uiassets, 9);
    // let city10 = spawn_cityentry_unowned(&mut commands, &uiassets, 10);
    // commands.entity(citylist_owned).push_children(&[city2, city3, city5, city6]);
    // commands.entity(citylist_unowned).push_children(&[city1, city4, city7, city8, city9, city10]);
    // commands.entity(citylist).push_children(&[citylist_owned, citylist_unowned]);
    commands.entity(topcenter).push_children(&[playerbar]);
    commands.entity(playerbar).push_children(&[player1, player2, player3, player4, player5, player6]);
    commands.entity(notify_area).push_children(&[notify_text]);
    commands.entity(minimap).push_children(&[minimap_image]);
    commands.entity(bottom).push_children(&[/*inventory, */minimap, bot_midarea]);
    commands.entity(bot_midarea).push_children(&[info_area, toolbar]);
    commands.entity(info_area).push_children(&[info_text]);
    commands.entity(toolbar).push_children(&[tool1, tool2, tool3, tool4]);
    // commands.entity(inventory).push_children(&[inventory_text, inventory_contents]);
    // commands.entity(inventory_contents).push_children(&[inventory_mines, inventory_decoys]);
    // commands.entity(inventory_mines).push_children(&[inventory_mines_icon, inventory_mines_text]);
    // commands.entity(inventory_decoys).push_children(&[inventory_decoys_icon, inventory_decoys_text]);
    commands.entity(root).push_children(&[/*citylist, */topcenter, notify_area, bottom]);
}

#[derive(Component)]
struct MinimapImageNode;

fn minimap_image_scale_fixme(
    mut q: Query<&mut Style, With<MinimapImageNode>>,
    minimap_image: Res<MinimapImage>,
    mut evr: EventReader<AssetEvent<Image>>,
    ass_image: Res<Assets<Image>>,
    uiscale: Res<UiScale>,
    q_wnd: Query<&Window, With<PrimaryWindow>>,
) {
    for ev in evr.iter() {
        if let AssetEvent::Modified { handle } = ev {
            if handle == &minimap_image.0 {
                let img = ass_image.get(&handle).unwrap();
                let wndscale = q_wnd.single().scale_factor();
                let w = img.texture_descriptor.size.width as f64;
                let h = img.texture_descriptor.size.height as f64;
                let w = w / uiscale.scale / wndscale;
                let h = h / uiscale.scale / wndscale;
                for mut style in &mut q {
                    style.width = Val::Px(w as f32);
                    style.height = Val::Px(h as f32);
                }
            }
        }
    }
}
