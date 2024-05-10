use bevy::ecs::system::{lifetimeless::{SQuery, SRes}, SystemParam};
use iyes_perf_ui::prelude::*;
use mw_app_core::map::{tile::TileOwner, GridCursor, GridCursorTileEntity};
use mw_common::{game::TileKind, grid::Pos, plid::PlayerId};

use crate::{net::NetInfo, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_perf_ui_entry_type::<PerfUiNetRtt>();
    app.add_perf_ui_entry_type::<PerfUiGridCursor>();
    app.add_perf_ui_entry_type::<PerfUiTileKind>();
    app.add_perf_ui_entry_type::<PerfUiTileOwner>();
}

/// Custom Perf UI entry to show Network Ping
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

/// Custom Perf UI entry to show Grid Cursor coordinates
#[derive(Component)]
pub struct PerfUiGridCursor {
    /// The label text to display, to allow customization
    pub label: String,
    /// Required to ensure the entry appears in the correct place in the Perf UI
    pub sort_key: i32,
}

impl Default for PerfUiGridCursor {
    fn default() -> Self {
        PerfUiGridCursor {
            label: String::new(),
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

/// Custom Perf UI entry to show Tile Kind under cursor
#[derive(Component)]
pub struct PerfUiTileKind {
    /// The label text to display, to allow customization
    pub label: String,
    /// Required to ensure the entry appears in the correct place in the Perf UI
    pub sort_key: i32,
}

impl Default for PerfUiTileKind {
    fn default() -> Self {
        PerfUiTileKind {
            label: String::new(),
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

/// Custom Perf UI entry to show Tile Kind under cursor
#[derive(Component)]
pub struct PerfUiTileOwner {
    /// The label text to display, to allow customization
    pub label: String,
    /// Required to ensure the entry appears in the correct place in the Perf UI
    pub sort_key: i32,
}

impl Default for PerfUiTileOwner {
    fn default() -> Self {
        PerfUiTileOwner {
            label: String::new(),
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

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

impl PerfUiEntry for PerfUiTileKind {
    type Value = TileKind;
    type SystemParam = (
        SQuery<&'static GridCursorTileEntity>,
        SQuery<&'static TileKind>,
    );

    fn label(&self) -> &str {
        if self.label.is_empty() {
            "Tile Kind"
        } else {
            &self.label
        }
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        (gcte, q): &mut <Self::SystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        gcte.get_single().ok()
            .and_then(|gcte| gcte.0)
            .and_then(|e| q.get(e).ok())
            .copied()
    }
}

impl PerfUiEntry for PerfUiTileOwner {
    type Value = PlayerId;
    type SystemParam = (
        SQuery<&'static GridCursorTileEntity>,
        SQuery<&'static TileOwner>,
    );

    fn label(&self) -> &str {
        if self.label.is_empty() {
            "Tile Owner"
        } else {
            &self.label
        }
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        (gcte, q): &mut <Self::SystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        gcte.get_single().ok()
            .and_then(|gcte| gcte.0)
            .and_then(|e| q.get(e).ok())
            .map(|o| o.0)
    }
}

impl PerfUiEntry for PerfUiGridCursor {
    type Value = Pos;
    type SystemParam = SQuery<&'static GridCursor>;

    fn label(&self) -> &str {
        if self.label.is_empty() {
            "Grid Cursor"
        } else {
            &self.label
        }
    }

    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn update_value(
        &self,
        crs: &mut <Self::SystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        crs.get_single().ok().and_then(|x| x.0)
    }

    fn format_value(
        &self,
        value: &Self::Value,
    ) -> String {
        format!("Y:{:>3} X:{:>3}", value.y(), value.x())
    }

    fn width_hint(&self) -> usize {
        11
    }
}
