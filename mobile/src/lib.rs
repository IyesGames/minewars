use mw_app::prelude::*;

mod ui;

#[bevy_main]
fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_engine::plugin);
    app.add_plugins(mw_app_core::plugin);
    app.add_plugins(mw_app_io::plugin);
    app.add_plugins(mw_app::plugin);
    app.add_plugins(mw_app_gfx2d::plugin);
    app.add_plugins(mw_app_gfx3d::plugin);
    app.add_plugins(mw_ui_common::plugin);
    app.add_plugins(mw_ui_desktop::plugin);
    app.add_plugins(mw_ui_mobile::plugin);
    #[cfg(target_os = "android")]
    app.add_plugins(mw_platform_android::plugin);
    #[cfg(target_os = "ios")]
    app.add_plugins(mw_platform_ios::plugin);
    app.add_plugins(mw_app_game_minesweeper::plugin);

    app.add_plugins((
        crate::ui::plugin,
    ));

    app.run();
}
