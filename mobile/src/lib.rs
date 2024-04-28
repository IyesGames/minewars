use mw_app::prelude::*;

mod haptic_android;

#[bevy_main]
fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_app::plugin);
    #[cfg(feature = "proprietary")]
    app.add_plugins(mw_app_proprietary::plugin);

    // mobile extras
    app.add_plugins((
        haptic_android::plugin,
    ));

    app.run();
}
