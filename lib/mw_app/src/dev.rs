use mw_common::game::event::GameEvent;
use mw_app_core::haptic::HapticEvent;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Last, (
        debug_progress
            .run_if(resource_exists::<ProgressCounter>)
            .after(iyes_progress::TrackedProgressSet),
        debug_gameevents,
        debug_hapticevents,
        debug_appstate,
    ));
}

fn debug_progress(counter: Res<ProgressCounter>) {
    let progress = counter.progress();
    let progress_full = counter.progress_complete();
    trace!(
        "Progress: {}/{}; Full Progress: {}/{}",
        progress.done,
        progress.total,
        progress_full.done,
        progress_full.total,
    );
}

fn debug_gameevents(
    mut evr: EventReader<GameEvent>,
) {
    for ev in evr.read() {
        trace!("{:?}", ev);
    }
}

fn debug_hapticevents(
    mut evr: EventReader<HapticEvent>,
) {
    for ev in evr.read() {
        trace!("{:?}", ev);
    }
}

fn debug_appstate(
    mut state: Res<State<AppState>>,
) {
    if state.is_changed() {
        trace!("AppState: {:?}", state.get());
    }
}
