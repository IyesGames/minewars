use bevy::ecs::system::{lifetimeless::SQuery, SystemParam};
use iyes_perf_ui::prelude::*;
use iyes_perf_ui::entry::PerfUiEntry;
use mw_app_core::map::GridCursorTileEntity;

use crate::{map::{Ass3dTileKind, Ass3dTileVariant, TileAss3d}, prelude::*};

pub(crate) fn plugin(app: &mut App) {
    app.add_perf_ui_simple_entry::<PerfUiAss3dTileKind>();
    app.add_perf_ui_simple_entry::<PerfUiAss3dTileVariant>();
}

/// Custom Perf UI entry to show Tile 3D Asset Kind
#[derive(Component)]
pub struct PerfUiAss3dTileKind {
    /// The label text to display, to allow customization
    pub label: String,
    /// Required to ensure the entry appears in the correct place in the Perf UI
    pub sort_key: i32,
}

impl Default for PerfUiAss3dTileKind {
    fn default() -> Self {
        PerfUiAss3dTileKind {
            label: String::new(),
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

/// Custom Perf UI entry to show Tile 3D Asset Kind
#[derive(Component)]
pub struct PerfUiAss3dTileVariant {
    /// The label text to display, to allow customization
    pub label: String,
    /// Required to ensure the entry appears in the correct place in the Perf UI
    pub sort_key: i32,
}

impl Default for PerfUiAss3dTileVariant {
    fn default() -> Self {
        PerfUiAss3dTileVariant {
            label: String::new(),
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

impl PerfUiEntry for PerfUiAss3dTileKind {
    type Value = Ass3dTileKind;
    type SystemParam = (
        SQuery<&'static GridCursorTileEntity>,
        SQuery<&'static TileAss3d>,
    );

    fn label(&self) -> &str {
        if self.label.is_empty() {
            "Tile 3D Asset Kind"
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
            .map(|o| o.kind)
    }

    fn width_hint(&self) -> usize {
        16
    }
}

impl PerfUiEntry for PerfUiAss3dTileVariant {
    type Value = (Ass3dTileVariant, u8, u8);
    type SystemParam = (
        SQuery<&'static GridCursorTileEntity>,
        SQuery<&'static TileAss3d>,
    );

    fn label(&self) -> &str {
        if self.label.is_empty() {
            "Tile 3D Asset Variant"
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
            .map(|o| (o.variant, o.rotation, o.subvariant[1]))
    }

    fn format_value(
        &self,
        (variant, rotation, subvariant): &Self::Value,
    ) -> String {
        let variant_str = format!("{:?}", variant);
        format!("{:<3}/{}/{:>3}", variant_str, rotation, subvariant)
    }

    fn width_hint(&self) -> usize {
        9
    }
}
