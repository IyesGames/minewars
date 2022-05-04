use crate::prelude::*;
use iyes_bevy_util::ui::{on_butt_interact, UiInactive};

use crate::assets::UiAssets;

pub mod mainmenu;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(iyes_bevy_util::ui::init_camera);
        app.add_system(butt_interact_visual);
        app.add_system(butts::exitapp.run_if(on_butt_interact::<butts::ExitApp>));
        #[cfg(feature = "dev")]
        app.add_system(butts::play_dev.run_if(on_butt_interact::<butts::PlayDev>));
    }
}

mod butts {
    use crate::prelude::*;
    use crate::{AppGlobalState, GameMode, StreamSource};
    use bevy::app::AppExit;

    #[derive(Component, Default)]
    pub struct ExitApp;

    pub fn exitapp(mut ev: EventWriter<AppExit>) {
        ev.send(AppExit);
    }

    #[cfg(feature = "dev")]
    #[derive(Component, Default)]
    pub struct PlayDev;

    #[cfg(feature = "dev")]
    pub fn play_dev(mut commands: Commands) {
        commands.insert_resource(NextState(AppGlobalState::InGame));
        commands.insert_resource(NextState(GameMode::Dev));
        commands.insert_resource(NextState(StreamSource::Local));
    }
}

fn spawn_button<B: Component + Default>(
    commands: &mut Commands,
    uiassets: &UiAssets,
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
    }).insert(B::default()).id();

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
