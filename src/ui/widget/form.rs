use crate::{prelude::*, locale::L10nKey, assets::UiAssets};

pub struct FormLine(pub String, pub Entity);

pub fn create_form_layout(
    commands: &mut Commands,
    settings: &AllSettings,
    uiassets: &UiAssets,
    lines: &[FormLine],
) -> Entity {
    let wrapper = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Stretch,
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    for line in lines {
        let line_container = commands.spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
        )).id();
        let label_container = commands.spawn((
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
        let widget_container = commands.spawn((
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
        let label_text = commands.spawn((
            L10nKey(line.0.clone()),
            TextBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: uiassets.font.clone(),
                        font_size: 24.0 * settings.ui.text_scale,
                        color: settings.ui.color_text.into(),
                    },
                ),
                ..Default::default()
            },
        )).id();
        commands.entity(label_container).push_children(&[label_text]);
        commands.entity(widget_container).push_children(&[line.1]);
        commands.entity(line_container).push_children(&[label_container, widget_container]);
        commands.entity(wrapper).push_children(&[line_container]);
    }

    wrapper
}
