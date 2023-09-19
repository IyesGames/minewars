use crate::prelude::*;
use crate::ui::tooltip::InfoAreaText;
use crate::ui::{UiCamera, UiRoot};
use crate::locale::L10nKey;
use crate::assets::UiAssets;

use super::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("menu_mainmenu", spawn_mainmenu);
        app.add_systems(OnEnter(AppState::MainMenu), setup_mainmenu_layout);
        app.add_systems(Update, (
            spawn_mainmenu
                .run_if(in_state(AppState::MainMenu))
                .run_if(rc_spawn_mainmenu),
        ));
    }
}

/// Create the toplevel screen layout for the MainMenu app state
///
/// This sets up the containers where different menus will display themselves.
fn setup_mainmenu_layout(
    mut commands: Commands,
    settings: Res<AllSettings>,
    uiassets: Res<UiAssets>,
) {
    commands.spawn((
        StateDespawnMarker,
        UiCamera,
        Camera2dBundle::default(),
    ));

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

    let main_area = commands.spawn((
        MenuContainer,
        NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let space_eater = commands.spawn((
        NodeBundle {
            style: Style {
                flex_grow: 1.0,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let info_bar = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let info_text = commands.spawn((
        InfoAreaText,
        L10nKey(String::new()),
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: uiassets.font2_light.clone(),
                    font_size: 24.0 * settings.ui.text_scale,
                    color: settings.ui.color_text,
                },
            ),
            ..Default::default()
        },
    )).id();

    let top_bar = spawn_top_bar(&mut commands, &settings, &uiassets);

    commands.entity(info_bar)
        .push_children(&[info_text]);

    commands.entity(root)
        .push_children(&[top_bar, main_area, space_eater, info_bar]);
}

/// Creates the contents of the main menu.
fn spawn_mainmenu(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
    settings: Res<AllSettings>,
    mut stack: ResMut<MenuStack>,
    logo: Res<crate::assets::TitleLogo>,
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
        title.0 = "".into();
    }
    stack.0.clear();
    stack.0.push("menu_mainmenu".into());

    let wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let img_logo = commands
        .spawn((ImageBundle {
            image: UiImage::new(logo.image.clone()),
            style: Style {
                flex_shrink: 0.0,
                flex_grow: 0.0,
                margin: UiRect::all(Val::Px(8.0)),
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let rows_wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let row1 = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let row2 = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let row3 = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let butt_playofficial = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new(),
        "menu-button-play-official",
        "menu-tooltip-play-official",
        PROPRIETARY,
    );
    let butt_watch = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new(),
        "menu-button-watch",
        "menu-tooltip-watch",
        true,
    );
    let butt_playlan = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new().cli("menu_lan_join"),
        "menu-button-play-lan",
        "menu-tooltip-play-lan",
        true,
    );
    let butt_playoffline = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new().cli("menu_offline"),
        "menu-button-offline",
        "menu-tooltip-offline",
        true,
    );
    let butt_settings = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new(),
        "menu-button-settings",
        "menu-tooltip-settings",
        true,
    );
    let butt_credits = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new(),
        "menu-button-credits",
        "menu-tooltip-credits",
        true,
    );
    let butt_exit = spawn_menu_butt(
        &mut commands,
        &*uiassets,
        &*settings,
        OnClick::new().cli("exit"),
        "menu-button-exit",
        "menu-tooltip-exit",
        true,
    );

    let rows = &[
        spawn_menu_row(&mut commands, &[butt_playofficial, butt_watch, butt_playlan]),
        spawn_menu_row(&mut commands, &[butt_playoffline]),
        spawn_menu_row(&mut commands, &[butt_settings, butt_credits, butt_exit]),
    ];

    #[cfg(feature = "dev")]
    {
        let butt_dev = spawn_menu_butt(
            &mut commands,
            &*uiassets,
            &*settings,
            OnClick::new().cli("devmode"),
            "menu-button-dev",
            "menu-tooltip-dev",
            true,
        );
        commands.entity(rows[1]).push_children(&[butt_dev]);
    }

    commands.entity(rows_wrapper).push_children(rows);
    commands.entity(wrapper).push_children(&[img_logo, rows_wrapper]);
    commands.entity(container).push_children(&[wrapper]);
}

/// Ensure the main menu is spawned whenever there is no other menu displayed
fn rc_spawn_mainmenu(
    q_container: Query<Option<Ref<Children>>, With<MenuContainer>>,
) -> bool {
    let Ok(children) = q_container.get_single() else {
        return false;
    };

    if let Some(children) = children {
        children.is_empty()
    } else {
        true
    }
}
