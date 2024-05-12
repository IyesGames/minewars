use mw_app_core::ui::UiCamera;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Menu),
        setup_menu_camera,
    );
}

fn setup_menu_camera(
    mut commands: Commands,
) {
    commands.spawn((
        MenuCleanup,
        UiCamera,
        Camera2dBundle::default(),
    ));
}
