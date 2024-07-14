use mw_common::net::*;

use crate::{prelude::*, settings::NetworkingSettings};

pub mod host;
pub mod client;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup,
        start_net
            .in_set(SetStage::Want(SettingsSyncSS))
    );
    app.add_systems(
        Update,
        setup_quic_endpoint
            .pipe(print_error("Could not set up QUIC endpoint"))
            .pipe(setup_endpoint_fail)
            .in_set(NeedsNetRuntimeSet)
            .run_if(not(resource_exists::<QuicEndpoint>))
    );
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeedsNetRuntimeSet;

#[derive(Resource)]
pub struct NetRuntime(pub tokio::runtime::Runtime);

#[derive(Resource)]
pub struct QuicEndpoint(pub Option<Arc<quinn::Endpoint>>);

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
            info!("Set up networking (tokio) runtime.");
        }
        Err(e) => {
            error!("Could not set up networking (tokio) runtime: {}", e);
        }
    }
}

fn setup_quic_endpoint(
    mut commands: Commands,
    mut task: Local<Option<tokio::sync::oneshot::Receiver<AnyResult<Arc<quinn::Endpoint>>>>>,
    rt: Res<NetRuntime>,
    settings: Settings,
) -> AnyResult<()> {
    if let Some(mut t) = task.take() {
        match t.try_recv() {
            Ok(endpoint) => {
                commands.insert_resource(QuicEndpoint(Some(endpoint?)));
                info!("Set up QUIC endpoint.");
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                *task = Some(t);
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                bail!("Error receiving value");
            }
        }
    } else {
        let s_net = settings.get::<NetworkingSettings>().unwrap();
        let my_addr = s_net.my_addr.parse::<std::net::SocketAddr>()
            .with_context(|| format!("Not a valid ip address + port: {:?}", s_net.my_addr))?;
        let server_settings = s_net.server_settings.clone();
        let client_settings = s_net.default_client_settings.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        *task = Some(rx);
        rt.0.spawn(async move {
            let _ = tx.send(create_quic_endpoint(
                my_addr, server_settings, client_settings
            ).await);
        });
    }
    Ok(())
}

fn setup_endpoint_fail(In(r): In<AnyResult<()>>, mut commands: Commands) {
    if r.is_err() {
        commands.insert_resource(QuicEndpoint(None));
    }
}

async fn create_quic_endpoint(
    my_addr: SocketAddr,
    server_settings: Option<ServerSettings>,
    client_settings: Option<ClientSettings>,
) -> AnyResult<Arc<quinn::Endpoint>> {
    let mut server_crypto = None;
    if let Some(s) = server_settings {
        let crypto = load_server_crypto(&s).await
            .context("Cannot load server crypto")?;
        server_crypto = Some(crypto);
    }
    let mut client_crypto = None;
    if let Some(s) = client_settings {
        let crypto = load_client_crypto(&s).await
            .context("Cannot load client crypto")?;
        client_crypto = Some(crypto);
    }
    setup_quic(my_addr, server_crypto, client_crypto)
        .map(|e| Arc::new(e))
}
