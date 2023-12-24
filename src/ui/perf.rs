use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

use crate::{prelude::*, assets::UiAssets};

use super::*;

pub struct PerfUiPlugin;

impl Plugin for PerfUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(AppState::AssetsLoading), setup_perfui);
        app.add_systems(Update, (fps_text_update_system, rtt_text_update_system));
    }
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct RttText;

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
    let text_rtt = commands.spawn((
        RttText,
        TextBundle {
            text: Text::from_sections([
                TextSection {
                    value: "Ping: ".into(),
                    style: TextStyle {
                        font: uiassets.font2_bold.clone(),
                        font_size: 16.0 * settings.ui.text_scale,
                        color: Color::WHITE,
                    }
                },
                TextSection {
                    value: "N/A".into(),
                    style: TextStyle {
                        font: uiassets.font2_bold.clone(),
                        font_size: 16.0 * settings.ui.text_scale,
                        color: Color::WHITE,
                    }
                },
            ]),
            ..Default::default()
        },
    )).id();
    commands.entity(root).push_children(&[text_fps, text_rtt]);
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
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

fn rtt_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    netinfo: Res<crate::net::NetInfo>,
    mut query: Query<&mut Text, With<RttText>>,
) {
    for mut text in &mut query {
        if let Some(rtt) = netinfo.rtt {
            let millis = rtt.as_secs_f64() * 1000.0;
            text.sections[1].value = format!("{millis:.2}");
            let fps = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
                .unwrap_or(0.0);
            text.sections[1].style.color = if fps == 0.0 {
                Color::WHITE
            } else {
                let frame_time = 1.0 / fps * 1000.0;
                let frame_time2 = frame_time * 2.0;
                if millis >= frame_time2 {
                    Color::RED
                } else if millis >= frame_time {
                    Color::rgb(
                        1.0,
                        ((frame_time2 - millis) / frame_time) as f32,
                        0.0,
                    )
                } else if millis >= 0.0 {
                    Color::rgb(
                        (1.0 - (frame_time - millis) / frame_time) as f32,
                        1.0,
                        0.0,
                    )
                } else {
                    Color::GREEN
                }
            };
        } else {
            if text.sections[1].value != "N/A" {
                text.sections[1].value = "N/A".into();
                text.sections[1].style.color = Color::WHITE;
            }
        }
    }
}
