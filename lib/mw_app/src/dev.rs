use bevy::reflect::{DynamicEnum, DynamicVariant};
use mw_common::game::event::GameEvent;

use crate::prelude::*;
use crate::haptic::{HapticEvent, HapticEventSS};

pub struct DevPlugin;

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("devmode", cli_devmode);
        app.register_clicommand_args("AppState", cli_appstate);
        app.add_systems(
            Last,
            debug_progress
                .run_if(resource_exists::<ProgressCounter>)
                .after(iyes_progress::TrackedProgressSet),
        );
        app.add_systems(Update, (
            debug_gameevents
                .in_set(SetStage::WantChanged(GameOutEventSS)),
            debug_hapticevents
                .in_set(SetStage::WantChanged(HapticEventSS)),
        ));
    }
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

/// Temporary function to use during development
///
/// If there is no proper code to set up a camera in a given app state (or whatever)
/// yet, use this to spawn a default 2d camera.
#[allow(dead_code)]
fn debug_setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        StateDespawnMarker,
    ));
}

fn cli_appstate(In(args): In<Vec<String>>, mut next: ResMut<NextState<AppState>>) {
    if args.len() != 1 {
        error!("\"appstate <Value>\"");
        return;
    }

    let dyn_state = DynamicEnum::new(&args[0], DynamicVariant::Unit);
    if let Some(state) = FromReflect::from_reflect(&dyn_state) {
        next.set(state);
    } else {
        error!("Invalid app state: {}", args[0]);
    }
}

fn cli_devmode(
    mut appstate: ResMut<NextState<AppState>>,
) {
    appstate.set(AppState::InGame);
}
