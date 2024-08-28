use bevy::ecs::system::{lifetimeless::{SQuery, SRes}, SystemParam};
use iyes_perf_ui::prelude::*;
use iyes_perf_ui::entry::PerfUiEntry;
use mw_app_core::map::{tile::TileOwner, GridCursor, GridCursorTileEntity};
use mw_common::{game::TileKind, grid::Pos, plid::PlayerId};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_perf_ui_simple_entry::<PerfUiGridCursor>();
    app.add_perf_ui_simple_entry::<PerfUiTileKind>();
    app.add_perf_ui_simple_entry::<PerfUiTileOwner>();
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
