use mw_app::prelude::*;

#[bevy_main]
fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_app_core::plugin);
    app.add_plugins(mw_app::plugin);
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
    app.run();
}
