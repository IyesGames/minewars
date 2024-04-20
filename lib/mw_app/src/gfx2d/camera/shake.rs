use glam::DVec2;
use mw_common::prelude::noise::NoiseFn;

use crate::gfx2d::Gfx2dModeSet;
use crate::prelude::*;
use crate::haptic::*;

use super::GameCamera;

pub struct Gfx2dCameraShakePlugin;

impl Plugin for Gfx2dCameraShakePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            setup_haptic_shake_camera_2d
                .in_set(Gfx2dModeSet::Any)
                .after(super::setup_game_camera_2d)
        );
        app.add_systems(Update, (
            haptic_camera_2d_manage_waves
                .in_set(SetStage::Want(HapticEventSS)),
            haptic_camera_2d_apply,
        )
            .in_set(Gfx2dModeSet::Any)
        );
    }
}

struct ShakerWave {
    noise: noise::Perlin,
    attack_secs: f32,
    hold_secs: f32,
    decay_secs: f32,
    amplitude: f32,
    frequency: f32,
    start: Duration,
    direction: DVec2,
}

#[derive(Component, Default)]
struct Camera2dShaker {
    waves: Vec<ShakerWave>,
}

fn setup_haptic_shake_camera_2d(
    world: &mut World,
) {
    let entities: Vec<_> = world.query_filtered::<Entity, With<GameCamera>>()
        .iter(world)
        .collect();

    for e in entities {
        world.spawn((
            SpatialBundle::default(),
            Camera2dShaker::default(),
        )).add_child(e);
    }
}

fn haptic_camera_2d_manage_waves(
    time: Res<Time>,
    settings: Res<AllSettings>,
    mut evr_haptic: EventReader<HapticEvent>,
    mut q_shaker: Query<&mut Camera2dShaker>,
) {
    let mut rng = rand::thread_rng();

    let now = time.elapsed();

    for mut shaker in q_shaker.iter_mut() {
        shaker.waves.retain(|wave| {
            let elapsed_secs = (now - wave.start).as_secs_f32();
            elapsed_secs < wave.attack_secs + wave.hold_secs + wave.decay_secs
        });
    }

    for ev in evr_haptic.read() {
        for mut shaker in q_shaker.iter_mut() {
            if let Some(waves) = settings.camera.shake_2d.get(&ev.kind) {
                for wave in waves.iter() {
                    shaker.waves.push(ShakerWave {
                        amplitude: wave.0,
                        frequency: wave.1,
                        attack_secs: wave.2,
                        hold_secs: wave.3,
                        decay_secs: wave.4,
                        start: now,
                        direction: DVec2::new(
                            rng.gen_range(-1.0 ..= 1.0),
                            rng.gen_range(-1.0 ..= 1.0),
                        ).normalize(),
                        noise: noise::Perlin::new(rng.gen()),
                    });
                }
            }
        }
    }
}

fn haptic_camera_2d_apply(
    time: Res<Time>,
    mut q_shaker: Query<(&Camera2dShaker, &mut Transform)>,
) {
    use interpolation::Ease;

    let now = time.elapsed();

    for (shaker, mut xf) in q_shaker.iter_mut() {
        let mut new_translation = DVec2::ZERO;

        for wave in &shaker.waves {
            let elapsed_secs = (now - wave.start).as_secs_f64();
            let ts_hold = wave.attack_secs as f64;
            let ts_decay = ts_hold + wave.hold_secs as f64;
            let ts_end = ts_decay + wave.decay_secs as f64;
            let d_attack = ts_hold;
            let d_decay = ts_end - ts_decay;
            let amplitude = if elapsed_secs > 0.0 && elapsed_secs < ts_hold {
                let t = elapsed_secs / d_attack;
                let t = t.quadratic_in_out();
                interpolation::lerp(&0.0, &(wave.amplitude as f64), &t)
            } else if elapsed_secs <= ts_decay {
                wave.amplitude as f64
            } else if elapsed_secs <= ts_end {
                let t = (elapsed_secs - ts_decay) / d_decay;
                let t = t.quadratic_in_out();
                interpolation::lerp(&(wave.amplitude as f64), &0.0, &t)
            } else {
                0.0
            };
            if amplitude <= 0.0 {
                continue;
            }
            let sample = wave.noise
                .get([elapsed_secs * wave.frequency as f64])
                * amplitude;
            new_translation += sample * wave.direction;
        }

        xf.translation = Vec3::new(
            new_translation.x as f32,
            new_translation.y as f32,
            xf.translation.z,
        );
    }
}
