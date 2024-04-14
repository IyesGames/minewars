use mw_common::game::event::GameEvent;

use crate::prelude::*;
use crate::GameEventSet;

use self::worker::{NetWorkerControl, host::HostSessionConfig};

mod worker;

pub struct NetClientPlugin;

impl Plugin for NetClientPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("host_connect_last", cli_host_connect_last);
        app.init_resource::<NetInfo>();
        app.configure_sets(Update, NeedsNetSet.run_if(resource_exists::<NetWorkerThread>));
        app.add_systems(Update, (
            net_manager,
            setup_networkerthread
                .run_if(resource_added::<AllSettings>)
                .run_if(not(resource_exists::<NetWorkerThread>)),
            net_gameevent.in_set(GameEventSet).in_set(NeedsNetSet),
            net_status.in_set(NeedsNetSet),
        ));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeedsNetSet;

#[derive(Resource, Default)]
pub struct NetInfo {
    pub rtt: Option<Duration>,
    pub server_addr: Option<SocketAddr>,
}

#[derive(Resource)]
struct GameEventChannel(RxMpscU<GameEvent>);

#[derive(Resource)]
struct NetControlChannel(TxBroadcast<worker::NetWorkerControl>);

#[derive(Resource)]
struct NetStatusChannel(RxMpscU<worker::NetWorkerStatus>);

#[derive(Resource)]
pub struct NetWorkerThread {
    jh: Option<std::thread::JoinHandle<()>>,
    shutdown: CancellationToken,
}

impl Drop for NetWorkerThread {
    fn drop(&mut self) {
        self.shutdown.cancel();
        self.jh.take().unwrap().join().ok();
    }
}

fn setup_networkerthread(mut commands: Commands, settings: Res<AllSettings>) {
    let (tx_control, rx_control) = tokio::sync::broadcast::channel(8);
    let (tx_status, rx_status) = tokio::sync::mpsc::unbounded_channel();
    let (tx_game_event, rx_game_event) = tokio::sync::mpsc::unbounded_channel();

    commands.insert_resource(NetControlChannel(tx_control));
    commands.insert_resource(NetStatusChannel(rx_status));
    commands.insert_resource(GameEventChannel(rx_game_event));

    if !settings.net.enabled {
        return;
    }

    let rt = ManagedRuntime::new();
    let shutdown = rt.token();

    let networker_channels = worker::Channels {
        shutdown: shutdown.clone(), rx_control, tx_status, tx_game_event,
    };

    let networker_settings = settings.net.worker.clone();

    match std::thread::Builder::new()
        .name("minewars-net-worker".into())
        .spawn(|| worker::main(rt, networker_settings, networker_channels))
    {
        Ok(jh) => {
            commands.insert_resource(NetWorkerThread {
                shutdown,
                jh: Some(jh),
            });
        },
        Err(e) => {
            error!("Could not create net worker thread! Error: {}", e);
        }
    }
}

fn net_status(
    mut chan: ResMut<NetStatusChannel>,
    mut netinfo: ResMut<NetInfo>,
) {
    while let Ok(status) = chan.0.try_recv() {
        use worker::NetWorkerStatus::*;
        netinfo.rtt = match status {
            RttReport(rtt) => Some(rtt),
            _ => None,
        };
    }
}

fn net_gameevent(
    mut chan: ResMut<GameEventChannel>,
    mut evw: EventWriter<GameEvent>,
) {
    while let Ok(ge) = chan.0.try_recv() {
        evw.send(ge);
    }
}

fn net_manager(
    mut commands: Commands,
    thread: Option<Res<NetWorkerThread>>,
) {
    if let Some(thread) = thread {
        if thread.jh.as_ref().unwrap().is_finished() {
            commands.remove_resource::<NetWorkerThread>();
            warn!("Networking shut down.");
        }
    }
    // TODO: try to bring networking back up again, under certain conditions
}

fn cli_host_connect_last(
    settings: Res<AllSettings>,
    chan: Res<NetControlChannel>,
) {
    let config = HostSessionConfig {
        addr: settings.net.last_host_addr,
        session_id: Some(settings.net.last_host_sessionid),
        server_name: None,
    };
    chan.0.send(NetWorkerControl::ConnectHost(config)).ok();
}
