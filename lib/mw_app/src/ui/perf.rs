use bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, ecs::system::{lifetimeless::SRes, SystemParam}};
use iyes_perf_ui::prelude::*;

use crate::{assets::UiAssets, net::NetInfo, prelude::*};

use super::*;

pub struct PerfUiPlugin;

impl Plugin for PerfUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(iyes_perf_ui::PerfUiPlugin);
        app.add_perf_ui_entry_type::<PerfUiNetRtt>();
        app.add_systems(Startup, setup_perfui);
        app.add_systems(Update, (
            toggle_perfui
                .before(iyes_perf_ui::PerfUiSet::Setup),
        ));
    }
}

fn setup_perfui(
    mut commands: Commands,
) {
    commands.spawn((
        PerfUiCompleteBundle::default(),
        PerfUiNetRtt::default(),
    ));
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
            commands.spawn((
                PerfUiCompleteBundle::default(),
                PerfUiNetRtt::default(),
            ));
        }
    }
}

/// Custom Perf UI entry to show the time since the last mouse click
#[derive(Component)]
pub struct PerfUiNetRtt {
    /// The label text to display, to allow customization
    pub label: String,
    /// Should we display units?
    pub display_units: bool,
    /// Highlight the value if it goes above this threshold
    pub threshold_highlight: Option<f32>,
    /// Support color gradients!
    pub color_gradient: ColorGradient,
    /// Width for formatting the string
    pub digits: u8,
    /// Precision for formatting the string
    pub precision: u8,
    /// Required to ensure the entry appears in the correct place in the Perf UI
    pub sort_key: i32,
}

impl Default for PerfUiNetRtt {
    fn default() -> Self {
        PerfUiNetRtt {
            label: String::new(),
            display_units: true,
            threshold_highlight: Some(50.0),
            color_gradient: ColorGradient::new_preset_gyr(10.0, 20.0, 40.0).unwrap(),
            digits: 3,
            precision: 2,
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

// Implement the trait for integration into the Perf UI
impl PerfUiEntry for PerfUiNetRtt {
    type Value = f64;
    type SystemParam = Option<SRes<NetInfo>>;

    fn label(&self) -> &str {
        if self.label.is_empty() {
            "Network Ping"
        } else {
            &self.label
        }
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        netinfo: &mut <Self::SystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        netinfo.as_mut()
            .and_then(|netinfo| netinfo.rtt)
            .map(|rtt| rtt.as_secs_f64() * 1000.0)
    }

    fn format_value(
        &self,
        value: &Self::Value,
    ) -> String {
        let mut s = iyes_perf_ui::utils::format_pretty_float(self.digits, self.precision, *value);
        if self.display_units {
            s.push_str(" ms");
        }
        s
    }

    fn width_hint(&self) -> usize {
        let w = iyes_perf_ui::utils::width_hint_pretty_float(self.digits, self.precision);
        if self.display_units {
            w + 3
        } else {
            w
        }
    }

    fn value_color(
        &self,
        value: &Self::Value,
    ) -> Option<Color> {
        self.color_gradient.get_color_for_value(*value as f32)
    }

    fn value_highlight(
        &self,
        value: &Self::Value,
    ) -> bool {
        self.threshold_highlight
            .map(|t| (*value as f32) > t)
            .unwrap_or(false)
    }
}
