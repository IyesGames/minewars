#![cfg_attr(feature = "release", windows_subsystem = "windows")]

use mw_app::prelude::*;

mod ui;

#[bevy_main]
fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_app_core::plugin);
    app.add_plugins(mw_app::plugin);
    app.add_plugins(mw_app_game_minesweeper::plugin);
    #[cfg(feature = "gfx2d")]
    app.add_plugins(mw_app_gfx2d::plugin);
    #[cfg(feature = "gfx3d")]
    app.add_plugins(mw_app_gfx3d::plugin);
    #[cfg(feature = "mw_ui_desktop")]
    app.add_plugins(mw_ui_desktop::plugin);
    #[cfg(feature = "mw_ui_mobile")]
    app.add_plugins(mw_ui_mobile::plugin);
    #[cfg(feature = "proprietary")]
    app.add_plugins(mw_app_proprietary::plugin);

    #[cfg(target_os = "windows")]
    app.add_plugins(mw_platform_windows::plugin);
    #[cfg(target_os = "macos")]
    app.add_plugins(mw_platform_macos::plugin);
    #[cfg(target_os = "linux")]
    app.add_plugins(mw_platform_linux::plugin);

    app.add_plugins((
        crate::ui::plugin,
    ));

    mw_app_core::settings_manager::early_load_settings(
        &mut app, &[SETTINGS_APP, SETTINGS_USER, SETTINGS_LOCAL]
    );

    app.run();
}
