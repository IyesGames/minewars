use crate::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            load_or_init_settings
                .run_if(not(resource_exists::<SettingsLoaded>())));
        app.add_systems(Update, (
            loadscreen_wait_settings.track_progress().run_if(in_state(AppState::AssetsLoading)),
        ));
    }
}

#[derive(Resource)]
pub struct SettingsLoaded;

fn load_or_init_settings(
    mut commands: Commands,
    mut iothread: Local<Option<std::thread::JoinHandle<AllSettings>>>,
) {
    if let Some(iothr) = iothread.take() {
        if iothr.is_finished() {
            let allsettings = iothr.join().unwrap_or(AllSettings::default());
            commands.insert_resource(allsettings);
            commands.insert_resource(SettingsLoaded);
            info!("Settings: ready.");
        } else {
            *iothread = Some(iothr);
        }
    } else {
        *iothread = Some(std::thread::spawn(move || {
            let dir = directories::ProjectDirs::from(
                "com", "IyesGames", "MineWars",
            ).map(|dirs| dirs.preference_dir().to_owned());

            let mut settings = AllSettings::default();
            if let Some(dir) = dir {
                let path = dir.join("settings.toml");
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        if let Ok(s) = std::str::from_utf8(&bytes) {
                            match toml::from_str(s) {
                                Ok(loaded) => {
                                    info!("Settings successfully loaded from: {:?}", path);
                                    settings = loaded;
                                },
                                Err(e) => {
                                    error!("Error parsing user prefs from TOML: {}", e);
                                    // if there was a problem with the file, early return,
                                    // we don't want to overwrite it
                                    return settings;
                                }
                            }
                        } else {
                            error!("User prefs file is not UTF-8!");
                        }
                    }
                    Err(e) => {
                        error!("Could not read settings from user prefs file: {}", e);
                    }
                }
                // write the settings to the file
                do_write_settings(&settings, &dir, &path);
            }
            settings
        }));
    }
}

pub fn write_settings(
    world: &World,
) {
    let settings = world.resource::<AllSettings>().clone();
    std::thread::spawn(move || {
        let dir = directories::ProjectDirs::from(
            "com", "IyesGames", "MineWars",
        ).map(|dirs| dirs.preference_dir().to_owned());

        if let Some(dir) = dir {
            let path = dir.join("settings.toml");
            do_write_settings(&settings, &dir, &path);
        }
    });
}

fn do_write_settings(
    settings: &AllSettings,
    dir: &std::path::Path,
    file: &std::path::Path,
) {
    let bytes = toml::to_string(&settings)
        .expect("Settings could not be serialized to toml!");
    if let Err(e) = std::fs::create_dir_all(dir) {
        error!("Failed to create user preferences directory: {}", e);
    }
    if let Err(e) = std::fs::write(file, bytes) {
        error!("Failed to write default settings to user prefs file: {}", e);
    }
    info!("Settings written to: {:?}", file);
}

fn loadscreen_wait_settings(
    settings: Option<Res<AllSettings>>,
) -> Progress {
    settings.is_some().into()
}

