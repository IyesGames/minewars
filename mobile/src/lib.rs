use mw_app::prelude::*;

mod haptic_android;

#[bevy_main]
fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_app::MinewarsAppPlugin);
    #[cfg(feature = "proprietary")]
    app.add_plugins(mw_app_proprietary::MinewarsProprietaryPlugin);

    // mobile extras
    app.add_plugins((
        haptic_android::HapticAndroidPlugin,
    ));

    app.run();
}
