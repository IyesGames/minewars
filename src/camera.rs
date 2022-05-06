use crate::assets::TileAssets;
use crate::map::MaxViewBounds;
use crate::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit, MouseMotion};
use bevy::render::render_resource::FilterMode;
use bevy_tweening::*;
use bevy_tweening::lens::*;

use crate::AppGlobalState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppGlobalState::InGame, setup_camera);
        app.add_exit_system(AppGlobalState::InGame, despawn_with_recursive::<CameraCleanup>);
        app.add_exit_system(AppGlobalState::AssetsLoading, setup_tile_sampler);
        app.add_system(
            camera_control_zoom_mousewheel
                .run_in_state(AppGlobalState::InGame)
        );
        app.add_system(
            camera_control_pan_mousedrag
                .run_in_state(AppGlobalState::InGame)
                .after(camera_control_zoom_mousewheel)
        );
    }
}

#[derive(Component)]
struct CameraCleanup;

#[derive(Component)]
struct GameCamera;

fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(ZoomLevel(0))
        .insert(CameraCleanup)
        .insert(GameCamera);
}

/// The current camera zoom level
///
/// This is the exponent; the camera scale will be set to 2^N.
#[derive(Component)]
pub struct ZoomLevel(usize);

/// Predefined camera zoom levels; scroll wheel jumps between these
///
/// This is the exponent; the camera scale will be set to 2^N.
static ZOOM_LEVELS: &'static [f32] = &[
    0.0, 0.5,
    1.0, 1.5, 1.75,
    2.0, 2.25, 2.5, 2.75,
    3.0, 3.5,
    4.0,
    5.0,
];

fn setup_tile_sampler(
    tiles: Res<TileAssets>,
    mut imgs: ResMut<Assets<Image>>,
) {
    let mut img = imgs.get_mut(&tiles.tiles).unwrap();
    img.sampler_descriptor.mag_filter = FilterMode::Linear;
    img.sampler_descriptor.min_filter = FilterMode::Linear;
    img.sampler_descriptor.mipmap_filter = FilterMode::Linear;
}

fn camera_control_zoom_mousewheel(
    mut commands: Commands,
    mut wheel: EventReader<MouseWheel>,
    mut q: Query<(Entity, &Transform, &mut ZoomLevel), With<GameCamera>>,
) {
    const JUMP_DUR: Duration = Duration::from_millis(125);

    let mut change = 0.0;

    // accumulate all events into one variable
    for ev in wheel.iter() {
        let delta = match ev.unit {
            MouseScrollUnit::Line => -ev.y,
            MouseScrollUnit::Pixel => unimplemented!(),
        };
        change += delta;
    }

    if change != 0.0 {
        let (e, xf, mut level) = q.single_mut();

        let change = change as isize;
        let mut l = level.0 as isize;
        l += change;
        l = l.clamp(0, ZOOM_LEVELS.len() as isize - 1);
        level.0 = l as usize;

        commands.entity(e).insert(Animator::new(Tween::new(
            EaseFunction::QuadraticOut,
            TweeningType::Once,
            JUMP_DUR,
            TransformScaleLens {
                start: xf.scale,
                end: Vec3::splat(ZOOM_LEVELS[level.0].exp2()),
            }
        )));

        // commands.entity(e).insert(xf_cur.ease_to(
        //     xf_tgt.0,
        //     EaseFunction::QuadraticOut,
        //     EasingType::Once { duration },
        // ));
    }
}

fn camera_control_pan_mousedrag(
    btn: Res<Input<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut q_camera: Query<(&mut Transform, &ZoomLevel), With<GameCamera>>,
    bounds: Option<Res<MaxViewBounds>>,
) {
    if btn.pressed(MouseButton::Right) {
        let mut delta = Vec2::ZERO;

        for ev in motion.iter() {
            delta += ev.delta;
        }

        if delta != Vec2::ZERO {
            let (mut cam, _) = q_camera.single_mut();
            cam.translation.x -= delta.x * cam.scale.x;
            cam.translation.y += delta.y * cam.scale.y;

            if let Some(bounds) = bounds {
                let mut cam_xy = cam.translation.truncate();
                let r = cam_xy.length();
                if r > bounds.0 {
                    cam_xy = cam_xy.normalize() * bounds.0;
                    cam.translation.x = cam_xy.x;
                    cam.translation.y = cam_xy.y;
                }
            }
        }
    }
    if btn.just_released(MouseButton::Right) {
        let (mut xf_cam, level) = q_camera.single_mut();
        // round camera translation to a full pixel at our current zoom level
        // (so rendering looks nice)
        xf_cam.translation.x = round_at_zoomlevel(level.0, xf_cam.translation.x);
        xf_cam.translation.y = round_at_zoomlevel(level.0, xf_cam.translation.y);
    }
}

fn round_at_zoomlevel(level: usize, x: f32) -> f32 {
    let levelscale = ZOOM_LEVELS[level].exp2();
    // round to zoom level scale
    let rounded = (x / levelscale).round() * levelscale;
    // round to whole pixel
    rounded.round()
}
