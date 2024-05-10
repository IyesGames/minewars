use bevy::ecs::query::{QueryData, QueryFilter, ROQueryItem};
use bevy::ecs::system::SystemParam;

use crate::prelude::*;

pub mod prelude {
    pub use super::{Settings, SettingsSyncSS};
}

pub fn plugin(app: &mut App) {
    app.configure_stage_set_no_rc(
        Update,
        SettingsSyncSS,
        // TODO: RC
    );
}

/// StageSet for settings load/store
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SettingsSyncSS;

/// Marker component for the settings governor entity
#[derive(Component)]
pub struct SettingsGovernor;

#[derive(SystemParam)]
pub struct Settings<'w, 's, T: QueryData + 'static, F: QueryFilter + 'static = ()> {
    query_settings: Query<'w, 's, T, (F, With<SettingsGovernor>)>,
}

impl<'w, 's, T: QueryData + 'static, F: QueryFilter + 'static> Settings<'w, 's, T, F> {
    pub fn get(&self) -> Option<ROQueryItem<'_, T>> {
        self.query_settings.get_single().ok()
    }
    pub fn get_mut(&mut self) -> Option<T::Item<'_>> {
        self.query_settings.get_single_mut().ok()
    }
}
