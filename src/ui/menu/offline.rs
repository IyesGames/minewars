use crate::prelude::*;
use crate::assets::UiAssets;

use super::*;

pub struct OfflineMenuPlugin;

impl Plugin for OfflineMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("menu_offline", spawn_menu_offline);
    }
}

fn spawn_menu_offline(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
    settings: Res<AllSettings>,
    mut stack: ResMut<MenuStack>,
    q_container: Query<Entity, With<MenuContainer>>,
    q_extras: Query<Entity, With<MenuTopBarExtras>>,
    mut q_title: Query<&mut L10nKey, With<MenuTitleText>>,
) {
    let Ok(container) = q_container.get_single() else {
        error!("Menu Container Entity not found!");
        return;
    };

    // clear any previous menu
    commands.entity(container).despawn_descendants();
    if let Ok(topbar) = q_extras.get_single() {
        commands.entity(topbar).despawn_descendants();
    }
    if let Ok(mut title) = q_title.get_single_mut() {
        title.0 = "menu-title-offline".into();
    }
    stack.0.push("menu_offline".into());

    let wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let butt_tutorial = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new(),
        "menu-button-play-tutorial",
        if PROPRIETARY { "menu-tooltip-play-tutorial" }
        else { "tooltip-unavailable-proprietary" },
        PROPRIETARY,
    );
    let butt_playground = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new().cli("offline_playground"),
        "menu-button-playground",
        if PROPRIETARY { "menu-tooltip-playground" }
        else { "tooltip-unavailable-proprietary" },
        PROPRIETARY,
    );
    let butt_replay = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new(),
        "menu-button-replay",
        "menu-tooltip-replay",
        true,
    );
    let butt_play_ms_single = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new().cli("minesweeper_singleplayer"),
        "menu-button-play-ms-single",
        "menu-tooltip-play-ms-single",
        true,
    );
    let butt_editor = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new(),
        "menu-button-editor",
        "menu-tooltip-editor",
        true,
    );

    let rows = &[
        spawn_menu_row(&mut commands, &[butt_tutorial]),
        spawn_menu_row(&mut commands, &[butt_playground]),
        spawn_menu_row(&mut commands, &[butt_replay]),
        spawn_menu_row(&mut commands, &[butt_play_ms_single]),
        spawn_menu_row(&mut commands, &[butt_editor]),
    ];
    commands.entity(wrapper).push_children(rows);
    commands.entity(container).push_children(&[wrapper]);
}
