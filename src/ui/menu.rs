use crate::{prelude::*, locale::L10nKey, assets::UiAssets, settings::NeedsSettingsSet};

use super::tooltip::InfoText;

mod lan;
mod mainmenu;
mod offline;

pub(super) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("menu_back", cli_menu_back);
        app.init_resource::<MenuStack>();
        app.add_plugins((
            mainmenu::MainMenuPlugin,
            offline::OfflineMenuPlugin,
            lan::LanMenuPlugin,
        ));
        app.add_systems(Update, (
            menu_butt_interact_visual.in_set(NeedsSettingsSet),
            butt_back_showhide.run_if(resource_changed::<MenuStack>())
        ));
    }
}

/// Marker for the area where top bar items / buttons can be placed when in a menu
#[derive(Component)]
struct MenuTopBar;

/// Marker for the area where a menu can display its main content
#[derive(Component)]
struct MenuContainer;

/// Marker for menu top bar title text
#[derive(Component)]
struct MenuTitleText;

/// Marker for menu top bar extras (rightside) area
#[derive(Component)]
struct MenuTopBarExtras;

/// Marker for menu buttons
#[derive(Component)]
struct MenuButton;

/// The Back Button
#[derive(Component)]
struct MenuBackButton;

/// Used for the "Back" button
///
/// Stores CLI strings to be run when clicked.
#[derive(Resource, Default)]
struct MenuStack(Vec<String>);

fn cli_menu_back(
    mut commands: Commands,
    mut stack: ResMut<MenuStack>,
) {
    // the topmost entry is the current menu, so we gotta pop twice
    stack.0.pop();
    if let Some(cli) = stack.0.pop() {
        commands.run_clicommand(&cli);
    }
}

fn butt_back_showhide(
    mut q_butt_back: Query<&mut Style, With<MenuBackButton>>,
    stack: Res<MenuStack>,
) {
    use bevy::prelude::Display;
    if stack.0.len() > 1 {
        for mut butt_style in &mut q_butt_back {
            butt_style.display = Display::Flex;
        }
    } else {
        for mut butt_style in &mut q_butt_back {
            butt_style.display = Display::None;
        }
    }
}

fn spawn_menu_butt(
    commands: &mut Commands,
    uiassets: &UiAssets,
    settings: &AllSettings,
    behavior: OnClick,
    text: &'static str,
    info_text: &'static str,
    enabled: bool,
) -> Entity {
    let color_init = if enabled {
        settings.ui.color_menu_button
    } else {
        settings.ui.color_menu_button_inactive
    };

    let color_text = if enabled {
        settings.ui.color_text
    } else {
        settings.ui.color_text_inactive
    };

    let butt = commands.spawn((
        MenuButton,
        behavior,
        InfoText(info_text.to_owned()),
        ButtonBundle {
            background_color: BackgroundColor(color_init.into()),
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                margin: UiRect::all(Val::Px(4.0)),
                flex_grow: 1.0,
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let text = commands.spawn((
        L10nKey(text.to_owned()),
        TextBundle {
            text: Text::from_section(
                text,
                TextStyle {
                    color: color_text.into(),
                    font_size: 32.0 * settings.ui.text_scale,
                    font: uiassets.font.clone(),
                },
            ),
            ..Default::default()
        },
    )).id();

    commands.entity(butt).push_children(&[text]);

    if !enabled {
        commands.entity(butt).insert(UiDisabled);
    }

    butt
}

fn menu_butt_interact_visual(
    settings: Res<AllSettings>,
    mut query: Query<(
        &Interaction, &mut BackgroundColor,
    ), (
        Changed<Interaction>, With<MenuButton>, Without<UiDisabled>,
    )>,
) {
    for (interaction, mut color) in query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(settings.ui.color_menu_button_selected.into());
            }
            Interaction::Hovered => {
                *color = BackgroundColor(settings.ui.color_menu_button_selected.into());
            }
            Interaction::None => {
                *color = BackgroundColor(settings.ui.color_menu_button.into());
            }
        }
    }
}

fn spawn_menu_row(
    commands: &mut Commands,
    children: &[Entity],
) -> Entity {
    commands.spawn((
        NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },
    )).push_children(children).id()
}

fn spawn_top_bar(
    commands: &mut Commands,
    settings: &AllSettings,
    uiassets: &UiAssets,
) -> Entity {
    let top_bar = commands.spawn((
        MenuTopBar,
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let leftside = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let rightside = commands.spawn((
        MenuTopBarExtras,
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let midside = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    let title_text = commands.spawn((
        MenuTitleText,
        L10nKey(String::new()),
        TextBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: uiassets.font_bold.clone(),
                    font_size: 40.0 * settings.ui.text_scale,
                    color: settings.ui.color_text.into(),
                },
            ),
            ..Default::default()
        },
    )).id();

    let butt_back = spawn_menu_butt(
        commands,
        uiassets,
        settings,
        OnClick::new().cli("menu_back"),
        "menu-button-back",
        "menu-tooltip-back",
        true,
    );
    commands.entity(butt_back).insert(MenuBackButton);

    commands.entity(leftside).push_children(&[butt_back]);
    commands.entity(midside).push_children(&[title_text]);
    commands.entity(top_bar).push_children(&[leftside, midside, rightside]);

    top_bar
}
