use mw_common::net::{load_client_crypto, setup_quic_client};

use crate::prelude::*;

use mw_app::settings::NetWorkerConfig;

use super::*;

pub mod auth;
pub mod host;

#[derive(Debug, Clone)]
pub enum NetWorkerControl {
    Disconnect,
    ConnectHost(host::HostSessionConfig),
}

#[derive(Debug)]
pub enum NetWorkerStatus {
    NetError(anyhow::Error),
    NetDisabled,
    RttReport(Duration),
    HostDisconnected,
    HostConnected,
    AuthDisconnected,
    AuthConnected,
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

async fn async_main(rt: ManagedRuntime, config: NetWorkerConfig, mut channels: Channels) {
    let endpoint = match setup(&config).await  {
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
                        rt.spawn(host::host_session(
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
