use bevy::tasks::IoTaskPool;
use mw_app_core::{graphics::*, map::*, settings::GraphicsStyleSettings};

use crate::{mwfile::{loader::{load_mwfile, MwFileLoaderSettings}, saver::{save_mwfile, MwFileSaverSettings}, MwMap}, prelude::*};

pub fn plugin(app: &mut App) {
    app.register_clicommand_args("save_map", save_map);
    app.register_clicommand_args("start_map_viewer", start_map_viewer);
}

fn save_map(
    In(args): In<Vec<String>>,
    q_map: Query<(&MapDescriptor, &MapDataOrig), With<MapGovernor>>,
) {
    let Ok((desc, orig)) = q_map.get_single() else {
        error!("Cannot save map: no map currently active.");
        return;
    };
    let Some(path) = args.first().cloned() else {
        error!("Cannot save map: please specify path!");
        return;
    };
    let mwmap = MwMap {
        topology: desc.topology,
        data: orig.clone(),
    };
    let settings = MwFileSaverSettings {
        save_replay: false,
        save_map_items: false,
        compress_map: true,
        compress_frames: true,
    };
    let rt = IoTaskPool::get();
    rt.spawn(async move {
        let r: AnyResult<()> = async {
            let mut file = async_fs::File::create(&path).await?;
            save_mwfile(&mut file, &settings, &mwmap, None).await?;
            file.sync_all().await?;
            Ok(())
        }.await;
        match r {
            Ok(_) => info!("Map saved to {:?}.", path),
            Err(e) => error!("Could not save map to {:?}: {:#}.", path, e),
        };
    }).detach();
}

fn start_map_viewer(
    In(args): In<Vec<String>>,
    mut commands: Commands,
    settings: Settings,
    mut state: ResMut<NextState<AppState>>,
) {
    let Some(path) = args.first().cloned() else {
        error!("Cannot load map: please specify path!");
        return;
    };

    let lsettings = MwFileLoaderSettings {
        load_replay: false,
        load_map_items: true,
        verify_checksums: true,
    };
    let rt = IoTaskPool::get();
    let p = path.clone();
    let r: Vec<AnyResult<MwMap>> = rt.scope(|s| s.spawn(async move {
        let mut file = async_fs::File::open(&p).await?;
        let (map, _) = load_mwfile(&mut file, &lsettings).await?;
        Ok(map)
    }));
    let mwmap = match r.into_iter().next() {
        Some(Ok(map)) => {
            info!("Map loaded from {:?}.", path);
            map
        },
        Some(Err(e)) => {
            error!("Could not load map from {:?}: {:#}.", path, e);
            return;
        },
        None => {
            error!("Could not load map from {:?}", path);
            return;
        }
    };

    commands.spawn(
        MapGovernorBundle::from_map_src(mwmap.topology, mwmap.data)
    );

    let s_gfx = settings.get::<GraphicsStyleSettings>().unwrap();
    let e_gov_gfx = commands.spawn((
        GraphicsGovernorBundle {
            cleanup: default(),
            marker: GraphicsGovernor,
            style: CurrentGraphicsStyle(s_gfx.game_preferred_style),
        },
    )).id();
    if s_gfx.game_enable_both_styles {
        commands.entity(e_gov_gfx).insert((
            Gfx2dEnabled,
            Gfx3dEnabled,
        ));
    } else {
        match s_gfx.game_preferred_style {
            GraphicsStyle::Gfx2d => commands.entity(e_gov_gfx)
                .insert(Gfx2dEnabled),
            GraphicsStyle::Gfx3d => commands.entity(e_gov_gfx)
                .insert(Gfx3dEnabled),
        };
    }

    state.set(AppState::GameLoading);
}
