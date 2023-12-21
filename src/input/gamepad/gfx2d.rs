use mw_app::camera::{GameCamera, CameraControlSet};

use crate::{prelude::*, gfx2d::Gfx2dSet};
use super::*;

pub struct Gfx2dGamepadInputPlugin;

impl Plugin for Gfx2dGamepadInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            joystick_pancamera,
        )
          .in_set(CameraControlSet)
          .in_set(Gfx2dSet::Any)
          .in_set(GameInputSet::Process)
          .run_if(rc_joystick_pancamera)
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

    let delta = get_joystick(*source, &axes)
        * settings.input.gamepad.pan_sens
        * time.delta_seconds();

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
