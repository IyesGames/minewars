use crate::prelude::*;
use bevy_asset_loader::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(AppState::StartupLoading)
    );
}
