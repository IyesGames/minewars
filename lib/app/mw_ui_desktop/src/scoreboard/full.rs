use mw_ui_common::root::spawn_root;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

fn spawn_full_scoreboard(
    mut commands: Commands,
) {
    let e_root = spawn_root(&mut commands, Style {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..Default::default()
    });
    let e_scoreboard = commands.spawn((
        NodeBundle {
            style: Style {
                display: bevy::ui::Display::Grid,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    commands.entity(e_root).add_child(e_scoreboard);
}

fn spawn_scoreboard_row(
    commands: &mut Commands,
    e_scoreboard: Entity,
    e_plid_icon: Entity,
    e_plid_stats: Entity,
    e_subplid_info: Entity,
    row_start: i16,
) {
    let e_wrap_plid_icon = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                grid_row: GridPlacement::start(row_start),
                grid_column: GridPlacement::start(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    commands.entity(e_wrap_plid_icon).add_child(e_plid_icon);
    commands.entity(e_scoreboard).add_child(e_wrap_plid_icon);
    let e_wrap_plid_stats = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                grid_row: GridPlacement::start(row_start),
                grid_column: GridPlacement::start(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    commands.entity(e_wrap_plid_stats).add_child(e_plid_stats);
    commands.entity(e_scoreboard).add_child(e_wrap_plid_stats);
    let e_wrap_subplid_info = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                grid_row: GridPlacement::start(row_start),
                grid_column: GridPlacement::start(1),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    commands.entity(e_wrap_subplid_info).add_child(e_subplid_info);
    commands.entity(e_scoreboard).add_child(e_wrap_subplid_info);
}
