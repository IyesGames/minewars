//! The User Governor
//!
//! The User Governor is the entity that carries all info about
//! the user of the app (i.e the actual human at the computer).
//!
//! This includes data about the profile / ID, authentication, etc.
//!
//! The User Governor should exist at all times, but various additional
//! components/state can be added to it later. A basic one is spawned
//! on app startup. After loading settings, connecting to servers, etc,
//! various state on it might be changed.

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<UserProfile>();
}

#[derive(Bundle)]
pub struct UserGovernorBundle {
    pub marker: UserGovernor,
    pub profile: MyUserProfile,
}

#[derive(Component)]
pub struct UserGovernor;

#[derive(Component)]
pub struct MyUserProfile(pub UserProfile);

#[derive(Reflect, Debug, Clone)]
pub struct UserProfile {
    pub display_name: String,
}
