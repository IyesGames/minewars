use mw_app_core::{camera::*, map::*};

use crate::{prelude::*, settings::Camera2dControlSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        jump_tween_start
            .in_set(SetStage::WantChanged(CameraJumpSS)),
        jump_tween
            .run_if(rc_jump_tween)
            .in_set(SetStage::Provide(CameraControlSS)),
    ).chain());
}

#[derive(Component, Default)]
pub struct CameraJumpTweenState {
    pub timer: Option<Timer>,
    curve: CubicSegment<Vec2>,
    start_translation: Vec2,
    end_translation: Vec2,
}

fn jump_tween_start(
    settings: Settings,
    mut evr_jump: EventReader<CameraJumpTo>,
    q_map: Query<&MapDescriptor, With<MapGovernor>>,
    mut q_camera: Query<(
        &mut CameraJumpTweenState, &Transform,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let desc = q_map.single();
    if let Some(ev) = evr_jump.read().last() {
        let pos_translation = match desc.topology {
            Topology::Hex => {
                let tdim = Vec2::new(crate::misc::sprite::WIDTH6, crate::misc::sprite::HEIGHT6);
                Hex::from(ev.0).translation() * tdim
            },
            Topology::Sq => {
                let tdim = Vec2::new(crate::misc::sprite::WIDTH4, crate::misc::sprite::HEIGHT4);
                Sq::from(ev.0).translation() * tdim
            },
        };
        for (mut tween, xf) in &mut q_camera {
            tween.start_translation = xf.translation.truncate();
            tween.end_translation = pos_translation;
            tween.timer = Some(Timer::new(
                Duration::from_secs_f32(s_input.jump_tween_duration),
                TimerMode::Once
            ));
            tween.curve = CubicSegment::new_bezier(
                s_input.jump_tween_curve.0,
                s_input.jump_tween_curve.1,
            );
        }
    }
}

fn rc_jump_tween(
    q_camera: Query<&CameraJumpTweenState, With<ActiveGameCamera>>,
) -> bool {
    q_camera.iter().any(|tween| tween.timer.is_some())
}

fn jump_tween(
    time: Res<Time>,
    mut q_camera: Query<(
        &mut CameraJumpTweenState, &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    for (mut tween, mut xf, proj) in &mut q_camera {
        let tween = &mut *tween;
        let Some(timer) = &mut tween.timer else {
            continue;
        };
        if timer.finished() {
            tween.timer = None;
            xf.translation.x = tween.end_translation.x;
            xf.translation.y = tween.end_translation.y;
            xf.translation.x = (xf.translation.x / proj.scale).round() * proj.scale;
            xf.translation.y = (xf.translation.y / proj.scale).round() * proj.scale;
            continue;
        }
        let fraction = timer.fraction();
        let t = tween.curve.ease(fraction);
        let translation = tween.start_translation.lerp(tween.end_translation, t);
        xf.translation.x = translation.x;
        xf.translation.y = translation.y;
        timer.tick(time.delta());
    }
}
