use crate::{graphics::GraphicsStyle, prelude::*, user::{MyUserProfile, UserGovernor, UserProfile}};

pub fn plugin(app: &mut App) {
    app.init_setting::<GraphicsStyleSettings>(SETTINGS_LOCAL.as_ref());
    app.init_setting::<UserProfileSettings>(SETTINGS_USER.as_ref());
    app.init_setting::<PlidColorSettings>(SETTINGS_USER.as_ref());
}

#[derive(Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct UserProfileSettings(pub UserProfile);

impl Setting for UserProfileSettings {
    fn apply(&self, world: &mut World) {
        let mut q = world.query_filtered::<&mut MyUserProfile, With<UserGovernor>>();
        let mut profile = q.single_mut(world);
        profile.0 = self.0.clone();
    }
}

impl Default for UserProfileSettings {
    fn default() -> Self {
        UserProfileSettings(UserProfile {
            display_name: "New Player".into(),
        })
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct GraphicsStyleSettings {
    pub game_enable_both_styles: bool,
    pub game_preferred_style: GraphicsStyle,
    pub editor_enable_both_styles: bool,
    pub editor_preferred_style: GraphicsStyle,
}

impl Default for GraphicsStyleSettings {
    fn default() -> Self {
        GraphicsStyleSettings {
            game_enable_both_styles: true,
            game_preferred_style: GraphicsStyle::Gfx2d,
            editor_enable_both_styles: true,
            editor_preferred_style: GraphicsStyle::Gfx2d,
        }
    }
}

impl Setting for GraphicsStyleSettings {}

#[derive(Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct PlidColorSettings {
    pub colors: Vec<Oklcha>,
    pub fog: Oklcha,
}

impl Default for PlidColorSettings {
    fn default() -> Self {
        PlidColorSettings {
            colors: vec![
                Oklcha::new(0.75, 0.0, 0.0, 1.0),
                Oklcha::new(0.5, 0.5, 0.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 11.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 6.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 3.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 13.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 8.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 2.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 12.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 4.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 14.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 7.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 1.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 9.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 5.0/15.0 * 360.0, 1.0),
                Oklcha::new(0.5, 0.5, 10.0/15.0 * 360.0, 1.0),
            ],
            fog: Oklcha::new(0.25, 0.0, 0.0, 1.0),
        }
    }
}

impl Setting for PlidColorSettings {}
