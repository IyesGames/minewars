use bevy::{input::mouse::{MouseWheel, MouseScrollUnit}, window::PrimaryWindow};
use bevy_tweening::*;
use mw_common::{game::MapDescriptor, grid::*};

use crate::{prelude::*, ui::UiCamera, input::GameInputSet};
use crate::{camera::*, input::InputAction};

use super::Gfx2dSet;

pub struct Gfx2dCameraPlugin;

impl Plugin for Gfx2dCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(iyes_bevy_extras::d2::WorldCursorPlugin);
        app.add_systems(OnEnter(AppState::InGame), setup_game_camera_2d.in_set(Gfx2dSet::Any));
        app.add_systems(Update, (
            camera_control_zoom_mousewheel,
            inputaction_zoom
                .in_set(GameInputSet::ProcessEvents),
        )
         .in_set(CameraControlSet)
         .in_set(Gfx2dSet::Any)
        );
        app.add_systems(Update, (
            grid_cursor
                .run_if(resource_changed::<WorldCursor>()),
        )
         .in_set(GridCursorSet)
         .in_set(Gfx2dSet::Any)
         .after(iyes_bevy_extras::d2::WorldCursorSet)
        );
        app.add_systems(Update, component_animator_system::<OrthographicProjection>);
    }
}

fn setup_game_camera_2d(
    world: &mut World,
) {
    let camera = Camera2dBundle::default();

    world.spawn((StateDespawnMarker, GameCamera, UiCamera, WorldCursorCamera, camera));
}

fn grid_cursor(
    crs_in: Res<WorldCursor>,
    mut crs_out: ResMut<GridCursor>,
    mapdesc: Res<MapDescriptor>,
) {
    match mapdesc.topology {
        Topology::Hex => {
            let tdim = Vec2::new(super::sprite::WIDTH6, super::sprite::HEIGHT6);
            let conv = bevy::math::Mat2::from_cols_array(
                &[tdim.x, 0.0, tdim.x * 0.5, tdim.y * 0.75]
            ).inverse();
            let adj = conv * crs_in.pos;
            let new = Hex::from_f32_clamped(adj.into());
            if new.ring() <= mapdesc.size {
                let new_pos = Pos::from(new);
                if crs_out.0 != new_pos {
                    crs_out.0 = new_pos;
                }
            }
        }
        Topology::Sq => {
            let tdim = Vec2::new(super::sprite::WIDTH4, super::sprite::HEIGHT4);
            let adj = crs_in.pos / tdim;
            let new = Sq::from_f32_clamped(adj.into());
            if new.ring() <= mapdesc.size {
                let new_pos = Pos::from(new);
                if crs_out.0 != new_pos {
                    crs_out.0 = new_pos;
                }
            }
        }
    };
}

struct ProjectionScaleLens {
    start: f32,
    end: f32,
}

impl Lens<OrthographicProjection> for ProjectionScaleLens {
    fn lerp(&mut self, target: &mut OrthographicProjection, ratio: f32) {
        let scale = self.start + (self.end - self.start) * ratio;
        target.scale = scale;
    }
}

fn inputaction_zoom(
    mut commands: Commands,
    settings: Res<AllSettings>,
    q_cam: Query<(Entity, &OrthographicProjection), With<GameCamera>>,
    q_wnd: Query<&Window, With<PrimaryWindow>>,
    mut evr_action: EventReader<InputAction>,
) {
    for ev in evr_action.read() {
        if let InputAction::ZoomCamera(lines) = ev {
            if *lines != 0.0 {
                let wnd = q_wnd.single();
                let (e_cam, proj) = q_cam.single();

                let newscale = ((proj.scale as f64 / wnd.scale_factor()).round() + *lines as f64).clamp(1.0, 8.0) * wnd.scale_factor();

                let dur = Duration::from_millis(settings.camera.zoom_tween_duration_ms as u64);
                let tween = Animator::new(Tween::new(
                    EaseFunction::QuadraticOut,
                    dur,
                    ProjectionScaleLens {
                        start: proj.scale,
                        end: newscale as f32,
                    }
                ));
                commands.entity(e_cam).insert(tween);
            }
        }
    }
}

fn camera_control_zoom_mousewheel(
    mut commands: Commands,
    settings: Res<AllSettings>,
    q_cam: Query<(Entity, &OrthographicProjection), With<GameCamera>>,
    q_wnd: Query<&Window, With<PrimaryWindow>>,
    mut wheel: EventReader<MouseWheel>,
    mut pixels: Local<f32>,
    mut oldpixels: Local<f32>,
) {
    if wheel.is_empty() && *oldpixels == *pixels {
        *pixels = 0.0;
        return;
    }

    *oldpixels = *pixels;

    let mut lines = 0.0;

    // accumulate all events into one variable
    for ev in wheel.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                lines -= ev.y;
            },
            MouseScrollUnit::Pixel => {
                *pixels += ev.y;
            },
        }
    }

    if *pixels > 32.0 {
        lines += 1.0;
        *pixels = 0.0;
    }
    if *pixels < -32.0 {
        lines -= 1.0;
        *pixels = 0.0;
    }

    // round fractional values
    // (can happen on platforms like macOS that try too hard to be fancy)
    // away from zero
    if lines > 0.0 {
        lines = lines.ceil();
    } else if lines < 0.0 {
        lines = lines.floor();
    }

    if lines != 0.0 {
        let wnd = q_wnd.single();
        let (e_cam, proj) = q_cam.single();

        let newscale = ((proj.scale as f64 / wnd.scale_factor()).round() + lines as f64).clamp(1.0, 8.0) * wnd.scale_factor();

        let dur = Duration::from_millis(settings.camera.zoom_tween_duration_ms as u64);
        let tween = Animator::new(Tween::new(
            EaseFunction::QuadraticOut,
            dur,
            ProjectionScaleLens {
                start: proj.scale,
                end: newscale as f32,
            }
        ));
        commands.entity(e_cam).insert(tween);
    }
}
