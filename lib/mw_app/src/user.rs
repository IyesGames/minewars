use mw_app_core::user::*;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup,
        setup_user_governor
            .in_set(SetStage::Prepare(SettingsSyncSS))
    );
}

fn setup_user_governor(world: &mut World) {
    world.spawn(UserGovernorBundle {
        marker: UserGovernor,
        profile: MyUserProfile(UserProfile {
            display_name: default()
        }),
    });
}
