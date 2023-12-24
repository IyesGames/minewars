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
    pub shutdown: CancellationToken,
    pub tx_game_event: TxMpscU<GameEvent>,
    pub tx_status: TxMpscU<NetWorkerStatus>,
    pub rx_control: RxBroadcast<NetWorkerControl>,
}

impl Channels {
    fn net_disable(&mut self) {
        self.tx_status.send(NetWorkerStatus::NetDisabled).ok();
        self.shutdown.cancel();
    }
}

impl Clone for Channels {
    fn clone(&self) -> Self {
        Channels {
            shutdown: self.shutdown.clone(),
            tx_game_event: self.tx_game_event.clone(),
            tx_status: self.tx_status.clone(),
            rx_control: self.rx_control.resubscribe(),
        }
    }
}

async fn setup(config: &NetWorkerConfig) -> AnyResult<quinn::Endpoint> {
    let crypto = load_client_crypto(
        &config.ca_cert,
        false,
        &[""], "",
    ).await?;
    let endpoint = setup_quic_client(crypto, "0.0.0.0:0".parse().unwrap())?;
    Ok(endpoint)
}

async fn connect_host(
    endpoint: &quinn::Endpoint,
    config: &HostSessionConfig,
) -> AnyResult<quinn::Connection> {
    info!("Connecting to Host: {}", config.addr);
    let connecting = endpoint.connect(config.addr, config.server_name())?;
    let connection = connecting.await?;
    Ok(connection)
}

async fn async_main(rt: ManagedRuntime, config: NetWorkerConfig, mut channels: Channels) {
    let mut endpoint = match setup(&config).await  {
        Ok(endpoint) => endpoint,
        Err(e) => {
            error!("Could not set up networking: {:#}", e);
            channels.tx_status.send(NetWorkerStatus::NetError(e)).ok();
            channels.net_disable();
            return;
        }
    };

    info!("Networking started.");

    let mut session_cancel = rt.child_token();

    loop {
        tokio::select! {
            _ = rt.listen_shutdown() => {
                break;
            }
            Ok(control) = channels.rx_control.recv() => {
                match control {
                    NetWorkerControl::ConnectHost(config) => {
                        session_cancel.cancel();
                        session_cancel = rt.child_token();
                        rt.spawn(host_session(
                            endpoint.clone(),
                            config,
                            channels.clone(),
                            session_cancel.clone(),
                        ));
                    }
                    NetWorkerControl::Disconnect => {
                        session_cancel.cancel();
                    }
                }
            }
        }
    }
}

async fn host_session(
    endpoint: quinn::Endpoint,
    config: HostSessionConfig,
    mut channels: Channels,
    cancel: CancellationToken,
) {
    let conn = tokio::select! {
        _ = cancel.cancelled() => {
            return;
        }
        r = connect_host(&endpoint, &config) => {
            match r {
                Ok(conn) => {
                    info!("Connected to Host Server!");
                    channels.tx_status.send(NetWorkerStatus::HostConnected).ok();
                    conn
                }
                Err(e) => {
                    error!("Could not connect to host: {}", e);
                    channels.tx_status.send(NetWorkerStatus::NetError(e)).ok();
                    return;
                }
            }
        }
    };

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                break;
            }
            e = conn.closed() => {
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

pub fn main(rt: ManagedRuntime, config: NetWorkerConfig, channels: Channels) {
    let tokrt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_name_fn(|| {
             "minewars-net-worker".into()
        })
        .build()
        .expect("Cannot create tokio runtime!");

    tokrt.block_on(async_main(rt, config, channels));
}
