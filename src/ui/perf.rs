use iyes_perf_ui::prelude::*;
use mw_app::{prelude::*, ui::perf::{PerfUiGridCursor, PerfUiNetRtt, PerfUiTileKind, PerfUiTileOwner}};
use mw_app_gfx3d::ui::perf::{PerfUiAss3dTileKind, PerfUiAss3dTileVariant};

pub fn plugin(app: &mut App) {
    app.add_plugins(iyes_perf_ui::PerfUiPlugin);
    app.add_systems(Update, (
        toggle_perfui
            .before(iyes_perf_ui::PerfUiSet::Setup),
    ));
    #[cfg(feature = "dev")]
    app.add_systems(Startup, setup_perfui);
}

#[derive(Bundle, Default)]
struct MwPerfUiBundle {
    root: PerfUiRoot,
    fps: PerfUiEntryFPS,
    fps_worst: PerfUiEntryFPSWorst,
    frametime: PerfUiEntryFrameTime,
    frametime_worst: PerfUiEntryFrameTimeWorst,
    rtt: PerfUiNetRtt,
    entity_count: PerfUiEntryEntityCount,
    cpu_usage: PerfUiEntryCpuUsage,
    mem_usage: PerfUiEntryMemUsage,
    window_mode: PerfUiEntryWindowMode,
    window_present_mode: PerfUiEntryWindowPresentMode,
    window_resolution: PerfUiEntryWindowResolution,
    window_scale_factor: PerfUiEntryWindowScaleFactor,
    cursor_position: PerfUiEntryCursorPosition,
    grid_cursor: PerfUiGridCursor,
    tile_kind: PerfUiTileKind,
    tile_owner: PerfUiTileOwner,
    ass3d_kind: PerfUiAss3dTileKind,
    ass3d_variant: PerfUiAss3dTileVariant,
}

impl MwPerfUiBundle {
    fn new() -> Self {
        let mut r = MwPerfUiBundle::default();
        r.root.background_color = Color::rgba(0.0, 0.0, 0.0, 0.75);
        r.root.default_value_color = Color::WHITE;
        r.root.err_color = Color::rgba(0.5, 0.5, 0.5, 0.5);
        r
    }
}

#[allow(dead_code)]
fn setup_perfui(
    mut commands: Commands,
) {
    commands.spawn(MwPerfUiBundle::new());
}

fn toggle_perfui(
    mut commands: Commands,
    q_root: Query<Entity, With<PerfUiRoot>>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    if kbd.just_pressed(KeyCode::F12) {
        if let Ok(e) = q_root.get_single() {
            // despawn the existing Perf UI
            commands.entity(e).despawn_recursive();
        } else {
            // create a simple Perf UI with default settings
            // and all entries provided by the crate:
            commands.spawn(MwPerfUiBundle::new());
        }
    }
}
