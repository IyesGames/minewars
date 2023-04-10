use crate::prelude::*;
use crate::ui;
use iyes_bevy_util::ui::butt_handler;
use crate::assets::UiAssets;

pub struct ErrorPlugin;

impl Plugin for ErrorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ErrorEvent>();
        app.add_system(spawn_error_dialogs);
        app.add_system(debug_error_dialog);
        app.add_system(butt_handler(butts::dismiss));
    }
}

pub struct ErrorEvent {
    error: AnyError,
    is_critical: bool,
}

#[derive(Component)]
struct ErrorDialog;

fn spawn_error_dialogs(
    mut commands: Commands,
    uiassets: Option<Res<UiAssets>>,
    mut evr: EventReader<ErrorEvent>,
) {
    let uiassets = if let Some(uiassets) = uiassets {
        uiassets
    } else {
        return;
    };

    for ev in evr.iter() {
        let heading_style = TextStyle {
            color: if ev.is_critical {
                Color::RED
            } else {
                Color::ORANGE
            },
            font: uiassets.font_bold.clone(),
            font_size: 32.0
        };
        let body_style = TextStyle {
            color: Color::WHITE,
            font: uiassets.font_light.clone(),
            font_size: 12.0
        };

        let wrapper = commands.spawn_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Px(0.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        }).insert(ErrorDialog).id();

        let top = commands.spawn_bundle(NodeBundle {
            color: UiColor(Color::rgb(0.5, 0.5, 0.5)),
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let header = commands.spawn_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Undefined),
                flex_grow: 0.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let body = commands.spawn_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Undefined),
                flex_grow: 1.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let butts = commands.spawn_bundle(NodeBundle {
            color: UiColor(Color::NONE),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Undefined),
                flex_grow: 0.0,
                ..Default::default()
            },
            ..Default::default()
        }).id();

        let header_text = commands.spawn_bundle(TextBundle {
            text: Text::from_section(
                if ev.is_critical {
                    "Unrecoverable Problem Occurred"
                } else {
                    "Problem Occurred"
                },
                heading_style,
            ),
            ..Default::default()
        }).id();

        let body_text = commands.spawn_bundle(TextBundle {
            text: Text::from_section(
                format!("{:#}", ev.error),
                body_style,
            ),
            ..Default::default()
        }).id();

        let butt_exit = ui::spawn_button(
            &mut commands, &*uiassets,
            ui::butts::ExitApp,
            "Exit Game", true
        );

        commands.entity(butts).push_children(&[butt_exit]);

        if !ev.is_critical {
            let butt_dismiss = ui::spawn_button(
                &mut commands, &*uiassets,
                butts::Dismiss,
                "Continue", true
            );
            commands.entity(butts).push_children(&[butt_dismiss]);
        }

        commands.entity(body).push_children(&[body_text]);
        commands.entity(header).push_children(&[header_text]);
        commands.entity(top).push_children(&[header, body, butts]);
        commands.entity(wrapper).push_children(&[top]);
    }
}

mod butts {
    use crate::prelude::*;
    use super::*;

    #[derive(Component, Default, Clone)]
    pub(super) struct Dismiss;

    pub(super) fn dismiss(
        _: In<Dismiss>,
        mut commands: Commands,
        q: Query<Entity, With<ErrorDialog>>,
    ) {
        // FIXME: implement a way to dismiss only the specific dialog, not all of them
        for e in q.iter() {
            commands.entity(e).despawn_recursive();
        }
    }
}

#[allow(dead_code)]
fn debug_error_dialog(
    kbd: Res<Input<KeyCode>>,
    mut evw: EventWriter<ErrorEvent>,
) {
    if kbd.just_pressed(KeyCode::F7) {
        evw.send(ErrorEvent {
            error: anyhow!("Blah blah"),
            is_critical: false,
        })
    }
}
