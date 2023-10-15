use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use crate::{prelude::*, assets::UiAssets};

use super::*;

pub struct PerfUiPlugin;

impl Plugin for PerfUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::AssetsLoading), setup_perfui);
        app.add_systems(Update, fps_text_update_system);
    }
}

#[derive(Component)]
struct FpsText;

fn setup_perfui(
    mut commands: Commands,
    settings: Res<AllSettings>,
    uiassets: Res<UiAssets>,
) {
    let root = commands.spawn((
        NodeBundle {
            background_color: BackgroundColor(Color::BLACK.with_a(0.5)),
            z_index: ZIndex::Global(9999),
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Auto,
                right: Val::Percent(1.),
                bottom: Val::Auto,
                top: Val::Percent(1.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    let text_fps = commands.spawn((
        FpsText,
        TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "FPS: ".into(),
                    style: TextStyle {
                        font: uiassets.font2_bold.clone(),
                        font_size: 16.0 * settings.ui.text_scale,
                        color: Color::WHITE,
                    }
                },
                TextSection {
                    value: "0".into(),
                    style: TextStyle {
                        font: uiassets.font2_bold.clone(),
                        font_size: 16.0 * settings.ui.text_scale,
                        color: Color::RED,
                    }
                },
            ]),
            ..Default::default()
        },
    )).id();
    commands.entity(root).push_children(&[text_fps]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
                text.sections[1].style.color = if value >= 120.0 {
                    Color::GREEN
                } else if value >= 60.0 {
                    Color::rgb(
                        (1.0 - (value - 60.0) / (120.0 - 60.0)) as f32,
                        1.0,
                        0.0,
                    )
                } else if value >= 30.0 {
                    Color::rgb(
                        1.0,
                        ((value - 30.0) / (60.0 - 30.0)) as f32,
                        0.0,
                    )
                } else {
                    Color::RED
                }
            }
        }
    }
}
