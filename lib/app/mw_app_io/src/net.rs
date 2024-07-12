use crate::{prelude::*, settings::NetworkingSettings};

pub mod host;
pub mod client;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup,
        start_net
            .in_set(SetStage::Want(SettingsSyncSS))
    );
}

#[derive(Resource)]
pub struct NetRuntime(pub tokio::runtime::Runtime);

fn start_net(
    mut commands: Commands,
    settings: Settings,
) {
    let s_net = settings.get::<NetworkingSettings>().unwrap();
    if !s_net.enabled {
        return;
    }
    match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(s_net.threads)
        .thread_name("minewars-net-worker")
        .enable_all()
        .build()
    {
        Ok(rt) => {
            commands.insert_resource(NetRuntime(rt));
        }
        Err(e) => {
            error!("Could not set up networking (tokio) runtime: {}", e);
        }
    }
}
