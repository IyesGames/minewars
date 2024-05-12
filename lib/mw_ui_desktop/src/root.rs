use mw_app_core::ui::{UiCamera, UiRoot};

use crate::{prelude::*, settings::DesktopUiSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        ui_root_resize.run_if(rc_ui_root_needs_resize),
    ));
}

fn ui_root_resize(
    settings: Settings,
    mut q_root: Query<&mut Style, With<UiRoot>>,
    q_cam: Query<&Camera, With<UiCamera>>,
) {
    let Some(size) = q_cam.get_single().ok()
        .and_then(|cam| cam.logical_viewport_size())
    else {
        return;
    };

    let Some(settings) = settings.get::<DesktopUiSettings>() else {
        return;
    };

    // detect ultrawide (anything > 16:9)
    let uw_width_threshold = size.y * 16.0 / 9.0;
    let uw_extra_width = size.x - uw_width_threshold;

    let width = uw_width_threshold + uw_extra_width * settings.ultrawide_use_extra_width_ratio;
    let width = width.min(size.x) * settings.underscan_ratio;
    let height = size.y * settings.underscan_ratio;

    let lr = ((size.x - width) / 2.0).floor();
    let tb = ((size.y - height) / 2.0).floor();

    for mut root_style in &mut q_root {
        root_style.left = Val::Px(lr);
        root_style.right = Val::Px(lr);
        root_style.top = Val::Px(tb);
        root_style.bottom = Val::Px(tb);
    }
}

/// Run condition for `ui_root_resize`
fn rc_ui_root_needs_resize(
    settings: Settings,
    q_cam: Query<&Camera, With<UiCamera>>,
    mut last_size: Local<Option<Vec2>>,
) -> bool {
    let Ok(camera) = q_cam.get_single() else {
        return false;
    };

    let viewport_size = camera.logical_viewport_size();
    if viewport_size.is_none() {
        return false;
    }

    let size_changed = viewport_size != *last_size;
    *last_size = viewport_size;

    let settings_changed = settings.is_changed::<DesktopUiSettings>();

    settings_changed || size_changed
}
