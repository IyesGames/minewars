fn main() {
    let mut app = mw_app::setup_bevy_app();
    app.add_plugins(mw_app::MinewarsAppPlugin);
    #[cfg(feature = "proprietary")]
    app.add_plugins(mw_app_proprietary::MinewarsProprietaryPlugin);
    app.run();
}
