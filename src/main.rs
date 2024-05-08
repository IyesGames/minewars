use mw_app::prelude::*;

#[bevy_main]
fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_app::plugin);
    app.add_plugins(mw_ui_desktop::plugin);
    app.add_plugins(mw_ui_mobile::plugin);
    #[cfg(feature = "proprietary")]
    app.add_plugins(mw_app_proprietary::plugin);
    app.run();
}
