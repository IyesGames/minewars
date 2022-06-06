use crate::prelude::*;
use iyes_bevy_util::ui::{butt_handler, UiInactive};

use crate::assets::UiAssets;

pub mod mainmenu;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(iyes_bevy_util::ui::init_camera);
        app.add_system(butt_interact_visual);
        // app.add_system(butts::exitapp.run_if(on_butt_interact::<butts::ExitApp>));
        app.add_system(butt_handler(butts::exitapp));
        app.add_system(butt_handler(butts::enter_game_mode));
    }
}

mod butts {
    use crate::prelude::*;
    use crate::{AppGlobalState, GameMode, StreamSource};
    use bevy::app::AppExit;

    #[derive(Component, Default, Clone)]
    pub struct ExitApp;

    pub fn exitapp(_: In<ExitApp>, mut ev: EventWriter<AppExit>) {
        ev.send(AppExit);
    }

    #[derive(Component, Clone)]
    pub struct EnterGameMode {
        pub mode: GameMode,
        pub source: StreamSource,
    }

    pub fn enter_game_mode(In(butt): In<EnterGameMode>, mut commands: Commands) {
        commands.insert_resource(NextState(AppGlobalState::GameLobby));
        commands.insert_resource(NextState(butt.mode));
        commands.insert_resource(NextState(butt.source));
    }

    #[derive(Component, Clone)]
    pub struct ShowCredits;

    pub fn show_credits(In(butt): In<ShowCredits>, mut commands: Commands) {
        unimplemented!()
    }

    #[derive(Component, Clone)]
    pub struct SettingsMenu;

    pub fn settings_menu(In(butt): In<ShowCredits>, mut commands: Commands) {
        unimplemented!()
    }
}

fn spawn_button<B: Component + Clone>(
    commands: &mut Commands,
    uiassets: &UiAssets,
    btn: B,
    text: &'static str,
    enabled: bool,
) -> Entity {
    let color_init = if enabled {
        Color::rgb(0.24, 0.24, 0.25)
    } else {
        Color::rgb(0.16, 0.15, 0.15)
    };

    let color_text = if enabled {
        Color::WHITE
    } else {
        Color::rgb(0.48, 0.44, 0.42)
    };

    let butt = commands.spawn_bundle(ButtonBundle {
        color: UiColor(color_init),
        style: Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: Rect::all(Val::Px(4.0)),
            margin: Rect::all(Val::Px(4.0)),
            flex_grow: 1.0,
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }).insert(btn).id();

    let text = commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            text,
            TextStyle {
                color: color_text,
                font_size: 32.0,
                font: uiassets.font_regular.clone(),
            },
            Default::default(),
        ),
        ..Default::default()
    }).id();

    commands.entity(butt).push_children(&[text]);

    if !enabled {
        commands.entity(butt).insert(UiInactive);
    }

    butt
}

fn butt_interact_visual(
    mut query: Query<(
        &Interaction, &mut UiColor,
    ), (
        Changed<Interaction>, With<Button>, Without<UiInactive>,
    )>,
) {
    for (interaction, mut color) in query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *color = UiColor(Color::rgb(0.24, 0.24, 0.25));
            }
            Interaction::Hovered => {
                *color = UiColor(Color::rgb(0.20, 0.20, 0.25));
            }
            Interaction::None => {
                *color = UiColor(Color::rgb(0.24, 0.24, 0.25));
            }
        }
    }
}
