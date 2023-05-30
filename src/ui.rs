use crate::{prelude::*, assets::UiAssets, locale::{L10nKey, L10nResolveSet}};

pub struct UiPlugin;

mod mainmenu;
mod hud;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(iyes_ui::UiExtrasPlugin);
        app.add_plugin(mainmenu::MainMenuPlugin);
        app.add_plugin(hud::HudPlugin);
        app.add_system(butt_interact_visual);
        app.add_system(butt_interact_infotext.before(L10nResolveSet));
    }
}

#[derive(Component)]
struct InfoAreaText;

#[derive(Component)]
struct InfoText(String);

fn spawn_button(
    commands: &mut Commands,
    uiassets: &UiAssets,
    behavior: OnClick,
    text: &'static str,
    info_text: &'static str,
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

    let butt = commands.spawn((
        behavior,
        InfoText(info_text.to_owned()),
        ButtonBundle {
            background_color: BackgroundColor(color_init),
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
                    color: color_text,
                    font_size: 32.0,
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

fn butt_interact_infotext(
    q_butt: Query<(&Interaction, &InfoText), Changed<Interaction>>,
    mut q_info: Query<&mut L10nKey, With<InfoAreaText>>,
) {
    let mut newtext = None;
    let mut clear = false;
    for (interaction, infotext) in &q_butt {
        match interaction {
            Interaction::None => {
                clear = true;
            }
            _ => {
                newtext = Some(&infotext.0);
            }
        }
    }
    if clear || newtext.is_some() {
        for mut infol10n in &mut q_info {
            if let Some(newtext) = newtext {
                infol10n.0 = String::from(newtext);
            } else {
                infol10n.0 = String::new();
            }
        }
    }
}

fn butt_interact_visual(
    mut query: Query<(
        &Interaction, &mut BackgroundColor,
    ), (
        Changed<Interaction>, With<Button>, Without<UiDisabled>,
    )>,
) {
    for (interaction, mut color) in query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                *color = BackgroundColor(Color::rgb(0.24, 0.24, 0.25));
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::rgb(0.20, 0.20, 0.25));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::rgb(0.24, 0.24, 0.25));
            }
        }
    }
}

