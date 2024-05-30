use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, window::PrimaryWindow};
use mw_app_core::{camera::{input::*, *}, graphics::{Gfx2dEnabled, GraphicsGovernor}, input::*, map::*};

use crate::{prelude::*, settings::Camera2dControlSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        input_analog_grid_cursor_motion
            .in_set(OnMouseMotionEventSet)
            .run_if(any_filter::<(With<AnalogGridCursor>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
    )
        .in_set(GameInputSet)
        .in_set(SetStage::Provide(GridCursorSS))
        .in_set(SetStage::Want(GameInputSS::Handle))
    );
}

fn input_analog_grid_cursor_motion(
    mut q_map: Query<(
        &mut GridCursor, &mut GridCursorTileEntity, &mut GridCursorTileTranslation,
        &MapDescriptor, &MapTileIndex,
    ), With<MapGovernor>>,
    q_camera: Query<(
        &Transform, &Camera,
    ), With<ActiveGameCamera>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let (mut crs, mut gcte, mut gctt, desc, index) = q_map.single_mut();
    let Ok((xf_camera, camera)) = q_camera.get_single() else {
        return;
    };
    let Ok(window) = q_window.get_single() else {
        return;
    };
    // assuming camera not affected by hierarchy
    let gxf_camera = GlobalTransform::from(*xf_camera);
    let Some(cursor) = window.cursor_position()
        .and_then(|pos| camera.viewport_to_world(&gxf_camera, pos))
        .map(|ray| ray.origin.truncate())
    else {
        crs.0 = None;
        gcte.0 = None;
        gctt.0 = None;
        return;
    };
    match desc.topology {
        Topology::Hex => {
            let tdim = Vec2::new(crate::misc::sprite::WIDTH6, crate::misc::sprite::HEIGHT6);
            let conv = bevy::math::Mat2::from_cols_array(
                &[tdim.x, 0.0, tdim.x * 0.5, tdim.y * 0.75]
            ).inverse();
            let adj = conv * cursor;
            let new = Hex::from_f32_clamped(adj.into());
            if new.ring() <= desc.size {
                let new_pos = Pos::from(new);
                if crs.0 != Some(new_pos) {
                    crs.0 = Some(new_pos);
                    gctt.0 = Some((new.translation() * tdim).extend(0.0));
                    if let Some(&new_e) = index.0.get(new_pos) {
                        gcte.0 = Some(new_e);
                    }
                }
            } else {
                crs.0 = None;
                gcte.0 = None;
                gctt.0 = None;
            }
        }
        Topology::Sq => {
            let tdim = Vec2::new(crate::misc::sprite::WIDTH4, crate::misc::sprite::HEIGHT4);
            let adj = cursor / tdim;
            let new = Sq::from_f32_clamped(adj.into());
            if new.ring() <= desc.size {
                let new_pos = Pos::from(new);
                if crs.0 != Some(new_pos) {
                    crs.0 = Some(new_pos);
                    gctt.0 = Some((new.translation() * tdim).extend(0.0));
                    if let Some(&new_e) = index.0.get(new_pos) {
                        gcte.0 = Some(new_e);
                    }
                }
            } else {
                crs.0 = None;
                gcte.0 = None;
                gctt.0 = None;
            }
        }
    }
}
