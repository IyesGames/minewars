use crate::{prelude::*, assets::UiAssets, ui, locale::L10nKey};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_mainmenu.in_schedule(OnEnter(AppState::MainMenu)));
        app.add_system(
            despawn_all_recursive::<With<MainMenuCleanup>>.in_schedule(OnExit(AppState::MainMenu)),
        );
    }
}

#[derive(Component)]
struct MainMenuCleanup;

fn setup_mainmenu(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
    logo: Res<crate::assets::TitleLogo>,
) {
    commands.spawn((MainMenuCleanup, Camera2dBundle::default()));

    let wrapper = commands
        .spawn((
            MainMenuCleanup,
            NodeBundle {
                background_color: BackgroundColor(Color::NONE),
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect::all(Val::Px(0.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .id();

    let top = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                flex_direction: FlexDirection::Column,
                min_size: Size::new(Val::Px(800.0), Val::Px(600.0)),
                size: Size::new(Val::Percent(75.0), Val::Percent(50.0)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let info_area = commands
        .spawn((NodeBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Px(32.0)),
                padding: UiRect {
                    top: Val::Auto,
                    bottom: Val::Px(4.0),
                    left: Val::Auto,
                    right: Val::Auto,
                },
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let info_text = commands
        .spawn((
            ui::InfoAreaText,
            L10nKey(String::new()),
            TextBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: uiassets.font2_light.clone(),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ),
                ..Default::default()
            },
        ))
        .id();

    commands.entity(info_area).push_children(&[info_text]);

    let img = commands
        .spawn((ImageBundle {
            image: UiImage::new(logo.image.clone()),
            style: Style {
                flex_shrink: 0.0,
                flex_grow: 0.0,
                margin: UiRect {
                    bottom: Val::Px(8.0),
                    ..UiRect::all(Val::Undefined)
                },
                align_self: AlignSelf::Center,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let row1 = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Undefined),
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let row2 = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Undefined),
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let row3 = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Undefined),
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let row4 = commands
        .spawn((NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Undefined),
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },))
        .id();

    let butt_playonline = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new(),
        "button-play-multiplayer",
        "tooltip-play-multiplayer",
        PROPRIETARY,
    );
    let butt_playlan = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new(),
        "button-play-lan",
        "tooltip-play-lan",
        PROPRIETARY,
    );
    let butt_playtutorial = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new(),
        "button-play-tutorial",
        "tooltip-play-tutorial",
        PROPRIETARY,
    );
    let butt_playoffline = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new(),
        "button-play-singleplayer",
        "tooltip-play-singleplayer",
        true,
    );
    let butt_watchlive = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new(),
        "button-watch-spectate",
        "tooltip-watch-spectate",
        true,
    );
    let butt_watchreplay = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new(),
        "button-watch-replay",
        "tooltip-watch-replay",
        true,
    );
    let butt_settings = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new(),
        "button-settings",
        "tooltip-settings",
        true,
    );
    let butt_exit = ui::spawn_button(
        &mut commands,
        &*uiassets,
        OnClick::new().cli("exit"),
        "button-exit",
        "tooltip-exit",
        true,
    );

    commands
        .entity(row1)
        .push_children(&[butt_playonline, butt_playlan]);
    commands
        .entity(row2)
        .push_children(&[butt_playtutorial, butt_playoffline]);
    commands
        .entity(row3)
        .push_children(&[butt_watchlive, butt_watchreplay]);
    commands
        .entity(row4)
        .push_children(&[butt_settings, butt_exit]);
    commands
        .entity(top)
        .push_children(&[img, row1, row2, row3, row4, info_area]);
    commands.entity(wrapper).push_children(&[top]);

    #[cfg(feature = "dev")]
    {
        let butt_dev = ui::spawn_button(
            &mut commands,
            &*uiassets,
            OnClick::new().cli("devmode"),
            "button-dev-play",
            "tooltip-dev-play",
            true,
        );
        commands.entity(row1).push_children(&[butt_dev]);
    }
}
