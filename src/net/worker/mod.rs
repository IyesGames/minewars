use mw_common::net::{load_client_crypto, setup_quic_client};

use crate::prelude::*;

use mw_app::settings::NetWorkerConfig;

use super::*;

#[derive(Debug, Clone)]
pub enum NetWorkerControl {
    Disconnect,
    ConnectHost(HostSessionConfig),
}

#[derive(Debug)]
pub enum NetWorkerStatus {
    NetError(anyhow::Error),
    NetDisabled,
    HostDisconnected,
    HostConnected,
    AuthDisconnected,
    AuthConnected,
}

#[derive(Debug, Clone)]
pub struct HostSessionConfig {
    pub addr: SocketAddr,
    pub session_id: Option<u32>,
    pub server_name: Option<String>,
}

impl HostSessionConfig {
    fn server_name(&self) -> &str {
        if let Some(name) = &self.server_name {
            name.as_str()
        } else {
            // FIXME
            "auth.iyes.games"
            // "mw_generic_host"
        }
    }
}

pub struct Channels {
    pub tx_shutdown: TxShutdown,
    pub rx_shutdown: RxShutdown,
    pub tx_game_event: TxMpscU<GameEvent>,
    pub tx_status: TxMpscU<NetWorkerStatus>,
    pub rx_control: RxBroadcast<NetWorkerControl>,
}

impl Channels {
    fn net_disable(&mut self) {
        self.tx_status.send(NetWorkerStatus::NetDisabled).ok();
        self.tx_shutdown.send(()).ok();
    }
}

impl Clone for Channels {
    fn clone(&self) -> Self {
        Channels {
            tx_shutdown: self.tx_shutdown.clone(),
            rx_shutdown: self.rx_shutdown.resubscribe(),
            tx_game_event: self.tx_game_event.clone(),
            tx_status: self.tx_status.clone(),
            rx_control: self.rx_control.resubscribe(),
        }
    }
}

struct HostSessionState {
    connection: quinn::Connection,
}

struct NetWorkerState {
    endpoint: quinn::Endpoint,
}

async fn setup(config: &NetWorkerConfig) -> AnyResult<NetWorkerState> {
    let crypto = load_client_crypto(
        &config.ca_cert,
        false,
        &[""], "",
    ).await?;
    let endpoint = setup_quic_client(crypto, "0.0.0.0:0".parse().unwrap())?;
    Ok(NetWorkerState {
        endpoint,
    })
}

async fn connect_host(
    endpoint: &quinn::Endpoint,
    config: &HostSessionConfig,
) -> AnyResult<HostSessionState> {
    info!("Connecting to Host: {}", config.addr);
    let connecting = endpoint.connect(config.addr, config.server_name())?;
    let connection = connecting.await?;
    Ok(HostSessionState {
        connection,
    })
}

async fn async_main(config: NetWorkerConfig, mut channels: Channels) {
    let mut state = match setup(&config).await  {
        Ok(state) => state,
        Err(e) => {
            error!("Could not set up networking: {}", e);
            channels.tx_status.send(NetWorkerStatus::NetError(e)).ok();
            channels.net_disable();
            return;
        }
    };

    loop {
        tokio::select! {
            _ = channels.rx_shutdown.recv() => {
                break;
            }
            Ok(control) = channels.rx_control.recv() => {
                match control {
                    NetWorkerControl::ConnectHost(config) => {
                        match connect_host(&state.endpoint, &config).await {
                            Ok(session) => {
                                channels.tx_status.send(NetWorkerStatus::HostConnected).ok();
                                info!("Connected to Host Server!");
                                host_session(&mut state, channels.clone(), session).await;
                            }
                            Err(e) => {
                                error!("Could not connect to host: {}", e);
                                channels.tx_status.send(NetWorkerStatus::NetError(e)).ok();
                            }
                        }
                    }
                    NetWorkerControl::Disconnect => {}
                }
            }
        }
    }
}

async fn host_session(wstate: &mut NetWorkerState, mut channels: Channels, session: HostSessionState) {
    loop {
        tokio::select! {
            _ = channels.rx_shutdown.recv() => {
                break;
            }
            e = session.connection.closed() => {
                match e {
                    quinn::ConnectionError::ApplicationClosed(_) => {}
                    _ => {
                        channels.tx_status.send(NetWorkerStatus::NetError(e.into())).ok();
                    }
                }
                break;
            }
            Ok(control) = channels.rx_control.recv() => {
                match control {
                    NetWorkerControl::Disconnect | NetWorkerControl::ConnectHost(_) => {
                        break;
                    }
                }
            }
        }
    }
    info!("Disconnected from Host Server!");
    channels.tx_status.send(NetWorkerStatus::HostDisconnected).ok();
}

pub fn main(config: NetWorkerConfig, channels: Channels) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name_fn(|| {
             "minewars-net-worker".into()
        })
        .build()
        .expect("Cannot create tokio runtime!");

    rt.block_on(async_main(config, channels));
}
