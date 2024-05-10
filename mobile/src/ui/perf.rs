#![allow(dead_code)]
#![allow(unused_variables)]

use iyes_perf_ui::prelude::*;
use mw_app::{prelude::*, ui::perf::PerfUiNetRtt};

pub fn plugin(app: &mut App) {
    #[cfg(feature = "dev")]
    app.add_plugins(iyes_perf_ui::PerfUiPlugin);
    #[cfg(feature = "dev")]
    app.add_systems(Startup, setup_perfui);
}

#[derive(Bundle, Default)]
struct MwPerfUiBundle {
    root: PerfUiRoot,
    fps: PerfUiEntryFPS,
    fps_worst: PerfUiEntryFPSWorst,
    rtt: PerfUiNetRtt,
    entity_count: PerfUiEntryEntityCount,
    cpu_usage: PerfUiEntryCpuUsage,
    mem_usage: PerfUiEntryMemUsage,
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

