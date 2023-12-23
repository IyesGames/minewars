use mw_app::camera::{GameCamera, CameraControlSet, GridCursorSet};
use mw_common::game::MapDescriptor;
use mw_common::grid::*;

use crate::{prelude::*, gfx2d::Gfx2dSet};
use super::*;

pub struct Gfx2dGamepadInputPlugin;

impl Plugin for Gfx2dGamepadInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            joystick_pancamera
                .run_if(rc_joystick_pancamera),
        )
          .in_set(CameraControlSet)
          .in_set(Gfx2dSet::Any)
          .in_set(GameInputSet::Process)
        );
        app.add_systems(Update, (
            joystick_gridcursormove
                .run_if(rc_joystick_gridcursormove),
        )
          .in_set(GridCursorSet)
          .in_set(Gfx2dSet::Any)
          .in_set(GameInputSet::Process)
        );
    }
}

fn rc_joystick_pancamera(
    analogs: Res<ActiveAnalogs>,
) -> bool {
    match analogs.0.get(&AnalogInput::PanCamera) {
        Some(&source) => source.is_gamepad(),
        None => false,
    }
}

fn joystick_pancamera(
    time: Res<Time>,
    settings: Res<AllSettings>,
    analogs: Res<ActiveAnalogs>,
    axes: Res<Axis<GamepadAxis>>,
    mut q_camera: Query<(&mut Transform, &OrthographicProjection), With<GameCamera>>,
    // bounds: Option<Res<MaxViewBounds>>,
) {
    let Some(source) = analogs.0.get(&AnalogInput::PanCamera) else {
        return;
    };

    let mut delta = get_joystick(*source, &axes);
    if settings.input.gamepad.pan_nonlinear {
        delta *= delta.length();
    }
    delta *= settings.input.gamepad.pan_sens;
    delta *= time.delta_seconds();

    // nonlinear feels better
    let delta = delta * delta.length();

    if delta != Vec2::ZERO {
        let (mut xf_cam, proj) = q_camera.single_mut();
        xf_cam.translation.x += delta.x * proj.scale;
        xf_cam.translation.y += delta.y * proj.scale;

/*
        if let Some(bounds) = bounds {
            let mut cam_xy = cam.translation.truncate();
            let r = cam_xy.length();
            if r > bounds.0 {
                cam_xy = cam_xy.normalize() * bounds.0;
                cam.translation.x = cam_xy.x;
                cam.translation.y = cam_xy.y;
            }
        }
*/
    }
    // if btn.just_released(MouseButton::Right) {
    //     let (mut xf_cam, _) = q_camera.single_mut();
    //     // round camera translation to a full pixel at our current zoom level
    //     // (so rendering looks nice)
    //     let xy = xf_cam.translation.truncate();
    //     xf_cam.translation = xy.round().extend(xf_cam.translation.z);
    // }
}

fn rc_joystick_gridcursormove(
    analogs: Res<ActiveAnalogs>,
) -> bool {
    match analogs.0.get(&AnalogInput::GridCursorMove) {
        Some(&source) => source.is_gamepad(),
        None => false,
    }
}

fn joystick_gridcursormove(
    time: Res<Time>,
    settings: Res<AllSettings>,
    analogs: Res<ActiveAnalogs>,
    axes: Res<Axis<GamepadAxis>>,
    mapdesc: Res<MapDescriptor>,
    mut crs_out: ResMut<GridCursor>,
    mut virtual_xy: Local<Vec2>,
) {
    let Some(source) = analogs.0.get(&AnalogInput::GridCursorMove) else {
        return;
    };

    let mut delta = get_joystick(*source, &axes);
    if settings.input.gamepad.gridcursor_nonlinear {
        delta *= delta.length();
    }
    delta *= settings.input.gamepad.gridcursor_sens;
    delta *= time.delta_seconds();

    if delta != Vec2::ZERO {
        match mapdesc.topology {
            Topology::Hex => {
                let tdim = Vec2::new(crate::gfx2d::sprite::WIDTH6, crate::gfx2d::sprite::HEIGHT6);
                let conv2 = bevy::math::Mat2::from_cols_array(
                    &[tdim.x, 0.0, tdim.x * 0.5, tdim.y * 0.75]
                );
                let conv = conv2.inverse();

                let adj = conv * *virtual_xy;
                let old = Hex::from_f32_clamped(adj.into());
                if crs_out.0 != old.into() {
                    virtual_xy.x = crs_out.0.x() as f32 + 0.5;
                    virtual_xy.y = crs_out.0.y() as f32 + 0.5;
                    *virtual_xy = conv2 * *virtual_xy;
                }

                *virtual_xy += delta;

                let adj = conv * *virtual_xy;
                let new = Hex::from_f32_clamped(adj.into());
                if new.ring() <= mapdesc.size {
                    let new_pos = Pos::from(new);
                    if crs_out.0 != new_pos {
                        crs_out.0 = new_pos;
                    }
                }
            }
            Topology::Sq => {
                let tdim = Vec2::new(crate::gfx2d::sprite::WIDTH4, crate::gfx2d::sprite::HEIGHT4);

                let adj = *virtual_xy / tdim;
                let old = Sq::from_f32_clamped(adj.into());
                if crs_out.0 != old.into() {
                    virtual_xy.x = crs_out.0.x() as f32 + 0.5;
                    virtual_xy.y = crs_out.0.y() as f32 + 0.5;
                    *virtual_xy *= tdim;
                }

                *virtual_xy += delta;

                let adj = *virtual_xy / tdim;
                let new = Sq::from_f32_clamped(adj.into());
                if new.ring() <= mapdesc.size {
                    let new_pos = Pos::from(new);
                    if crs_out.0 != new_pos {
                        crs_out.0 = new_pos;
                    }
                }
            }
        }
    }
}
