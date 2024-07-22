use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

/// Marker for UI root entities / top-level containers
#[derive(Component)]
pub struct UiRoot;

pub fn spawn_root(
    commands: &mut Commands,
    style_base: Style,
) -> Entity {
    commands.spawn((
        UiRoot,
        NodeBundle {
            background_color: BackgroundColor(Color::NONE),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..style_base
            },
            ..Default::default()
        },
    )).id()
}
