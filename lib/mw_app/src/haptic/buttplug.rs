//! Mapping of game haptics to buttplug
//!
//! The actual Buttplug I/O is handled by the MineWars Network
//! Worker thread (the tokio runtime environment) concurrently
//! with the game's netcode. Here we just interface with the
//! async tasks running there, via channels.

use crate::prelude::*;

use super::HapticEvent;

pub fn plugin(app: &mut App) {
}

#[derive(Component)]
struct ButtplugDevice {
}

fn vibrate_buttplug(
    mut evr_haptic: EventReader<HapticEvent>,
    mut q_device: Query<&mut ButtplugDevice>,
) {
}
