use bevy::input::mouse::MouseMotion;

use mw_app::camera::{GameCamera, CameraControlSet};

use crate::{prelude::*, gfx2d::Gfx2dSet};
use super::*;

pub struct Gfx2dMouseInputPlugin;

impl Plugin for Gfx2dMouseInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            mousemotion_pancamera,
        )
          .in_set(CameraControlSet)
          .in_set(Gfx2dSet::Any)
          .in_set(GameInputSet::Process)
          .run_if(rc_mousemotion_pancamera)
        );
    }
}

fn rc_mousemotion_pancamera(
    analogs: Res<ActiveAnalogs>,
) -> bool {
    analogs.0.get(&AnalogInput::PanCamera) == Some(&AnalogSource::MouseMotion)
}

fn mousemotion_pancamera(
    mut motion: EventReader<MouseMotion>,
    mut q_camera: Query<(&mut Transform, &OrthographicProjection), With<GameCamera>>,
    // bounds: Option<Res<MaxViewBounds>>,
) {
    let mut delta = Vec2::ZERO;

    for ev in motion.iter() {
        delta += ev.delta;
    }

    if delta != Vec2::ZERO {
        let (mut xf_cam, proj) = q_camera.single_mut();
        xf_cam.translation.x -= delta.x * proj.scale;
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
