use mw_app::prelude::*;

mod ui;

#[bevy_main]
fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_engine::plugin);
    app.add_plugins(mw_app_core::plugin);
    app.add_plugins(mw_app::plugin);
    app.add_plugins(mw_app_io::plugin);
    app.add_plugins(mw_app_game_minesweeper::plugin);

    #[cfg(feature = "gfx2d")]
    app.add_plugins(mw_app_gfx2d::plugin);
    #[cfg(feature = "gfx3d")]
    app.add_plugins(mw_app_gfx3d::plugin);
    #[cfg(any(feature = "mw_ui_desktop", feature = "mw_ui_mobile"))]
    app.add_plugins(mw_ui_common::plugin);
    #[cfg(feature = "mw_ui_desktop")]
    app.add_plugins(mw_ui_desktop::plugin);
    #[cfg(feature = "mw_ui_mobile")]
    app.add_plugins(mw_ui_mobile::plugin);

    #[cfg(feature = "proprietary")]
    app.add_plugins(mw_app_proprietary::plugin);

    #[cfg(target_os = "android")]
    app.add_plugins(mw_platform_android::plugin);
    #[cfg(target_os = "ios")]
    app.add_plugins(mw_platform_ios::plugin);

    app.add_plugins((
        crate::ui::plugin,
    ));

    app.run();
}
