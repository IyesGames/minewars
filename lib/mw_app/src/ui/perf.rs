use bevy::ecs::system::{lifetimeless::{SQuery, SRes}, SystemParam};
use iyes_perf_ui::prelude::*;
use mw_common::{game::TileKind, grid::Pos, plid::PlayerId};

use crate::{camera::GridCursor, gfx3d::map::{Ass3dTileKind, Ass3dTileVariant, TileAss3d}, map::{GridCursorTileEntity, TileOwner}, net::NetInfo, prelude::*};

pub struct PerfUiPlugin;

impl Plugin for PerfUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(iyes_perf_ui::PerfUiPlugin);
        app.add_perf_ui_entry_type::<PerfUiNetRtt>();
        app.add_perf_ui_entry_type::<PerfUiGridCursor>();
        app.add_perf_ui_entry_type::<PerfUiTileKind>();
        app.add_perf_ui_entry_type::<PerfUiTileOwner>();
        app.add_perf_ui_entry_type::<PerfUiAss3dTileKind>();
        app.add_perf_ui_entry_type::<PerfUiAss3dTileVariant>();
        app.add_systems(Update, (
            toggle_perfui
                .before(iyes_perf_ui::PerfUiSet::Setup),
        ));
        #[cfg(feature = "dev")]
        app.add_systems(Startup, setup_perfui);
    }
}

#[derive(Bundle, Default)]
struct MwPerfUiBundle {
    base: PerfUiCompleteBundle,
    grid_cursor: PerfUiGridCursor,
    tile_kind: PerfUiTileKind,
    tile_owner: PerfUiTileOwner,
    ass3d_kind: PerfUiAss3dTileKind,
    ass3d_variant: PerfUiAss3dTileVariant,
    rtt: PerfUiNetRtt,
}

impl MwPerfUiBundle {
    fn new() -> Self {
        let mut r = MwPerfUiBundle::default();
        r.base.root.background_color = Color::rgba(0.0, 0.0, 0.0, 0.75);
        r.base.root.default_value_color = Color::WHITE;
        r.base.root.err_color = Color::rgba(0.5, 0.5, 0.5, 0.5);
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
        SRes<GridCursorTileEntity>,
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
        gcte.0
            .and_then(|e| q.get(e).ok())
            .copied()
    }
}

impl PerfUiEntry for PerfUiTileOwner {
    type Value = PlayerId;
    type SystemParam = (
        SRes<GridCursorTileEntity>,
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
        gcte.0
            .and_then(|e| q.get(e).ok())
            .map(|o| o.0)
    }
}

impl PerfUiEntry for PerfUiAss3dTileKind {
    type Value = Ass3dTileKind;
    type SystemParam = (
        SRes<GridCursorTileEntity>,
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
        gcte.0
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
        SRes<GridCursorTileEntity>,
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
        gcte.0
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

impl PerfUiEntry for PerfUiGridCursor {
    type Value = Pos;
    type SystemParam = SRes<GridCursor>;

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
        Some(crs.0)
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
