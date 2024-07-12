use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<NetworkingSettings>(SETTINGS_LOCAL.as_ref());
}

#[derive(Reflect, Clone, PartialEq)]
#[reflect(Setting)]
pub struct NetworkingSettings {
    pub enabled: bool,
    pub threads: usize,
}

impl Setting for NetworkingSettings {}

impl Default for NetworkingSettings {
    fn default() -> Self {
        let threads = if num_cpus::get_physical() > 4 {
            4
        } else {
            2
        };
        Self {
            enabled: true,
            threads,
        }
    }
}
