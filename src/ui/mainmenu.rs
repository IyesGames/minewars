use crate::prelude::*;

use crate::AppGlobalState;
use crate::assets::UiAssets;
use crate::ui;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppGlobalState::MainMenu, setup);
        app.add_exit_system(AppGlobalState::MainMenu, despawn_with::<MainMenuCleanup>);
    }
}

#[derive(Component)]
struct MainMenuCleanup;

fn setup(
    mut commands: Commands,
    uiassets: Res<UiAssets>,
    logo: Res<crate::assets::TitleLogo>,
) {
    let wrapper = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect::all(Val::Px(0.0)),
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        ..Default::default()
    }).insert(MainMenuCleanup).id();

    let top = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
            min_size: Size::new(Val::Px(800.0), Val::Px(600.0)),
            size: Size::new(Val::Percent(75.0), Val::Percent(50.0)),
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let spacer = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Undefined, Val::Percent(25.0)),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let img = commands.spawn_bundle(ImageBundle {
        image: UiImage(logo.image.clone()),
        style: Style {
            flex_shrink: 0.0,
            flex_grow: 0.0,
            align_self: AlignSelf::Center,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let row1 = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Undefined),
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let row2 = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Undefined),
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let row3 = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Undefined),
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let row4 = commands.spawn_bundle(NodeBundle {
        color: UiColor(Color::NONE),
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Undefined),
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let butt_playonline = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Play ONLINE!", PROPRIETARY);
    let butt_playlan = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Play (LAN)", PROPRIETARY);
    let butt_playtutorial = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Tutorial", PROPRIETARY);
    let butt_playoffline = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Offline Practice", PROPRIETARY);
    let butt_watchlive = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Watch Live Game", true);
    let butt_watchreplay = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Watch Saved Replay", true);
    let butt_settings = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Settings", true);
    let butt_credits = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Credits", true);
    let butt_exit = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "Exit Game", true);

    commands.entity(row1).push_children(&[butt_playonline, butt_playlan]);
    commands.entity(row2).push_children(&[butt_playtutorial, butt_playoffline]);
    commands.entity(row3).push_children(&[butt_watchlive, butt_watchreplay]);
    commands.entity(row4).push_children(&[butt_settings, butt_credits, butt_exit]);
    commands.entity(top).push_children(&[img, spacer, row1, row2, row3, row4]);
    commands.entity(wrapper).push_children(&[top]);

    #[cfg(feature = "dev")]
    {
        let butt_dev = ui::spawn_button::<ui::butts::ExitApp>(&mut commands, &*uiassets, "(dev/debug mode)", true);
        commands.entity(row1).push_children(&[butt_dev]);
    }
}
