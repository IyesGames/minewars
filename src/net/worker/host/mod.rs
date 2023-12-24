use mw_proto_host::client::ProtoState;

use crate::prelude::*;
use super::*;

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

async fn connect_host(
    endpoint: &quinn::Endpoint,
    config: &HostSessionConfig,
) -> AnyResult<quinn::Connection> {
    info!("Connecting to Host: {}", config.addr);
    let connecting = endpoint.connect(config.addr, config.server_name())?;
    let connection = connecting.await?;
    Ok(connection)
}

pub(super) async fn host_session(
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

    channels.tx_status.send(NetWorkerStatus::RttReport(conn.rtt())).ok();

    let mut buf_tx = Vec::with_capacity(64);
    let mut buf_rx = vec![0; 512];
    let mut proto = ProtoState::new();

    loop {
        let mut rx_control = channels.rx_control.resubscribe();
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
            Ok(control) = rx_control.recv() => {
                match control {
                    NetWorkerControl::Disconnect | NetWorkerControl::ConnectHost(_) => {
                        break;
                    }
                }
            }
            r = drive_host_session(&config, &channels, &conn, &mut proto, &mut buf_tx, &mut buf_rx) => {
                channels.tx_status.send(NetWorkerStatus::RttReport(conn.rtt())).ok();
                if let Err(e) = r {
                    error!("Player<->Host Protocol Error: {:#}", e);
                    break;
                }
            }
        }
    }

    info!("Disconnected from Host Server!");
    channels.tx_status.send(NetWorkerStatus::HostDisconnected).ok();
}

async fn drive_host_session(
    config: &HostSessionConfig,
    channels: &Channels,
    conn: &quinn::Connection,
    proto: &mut ProtoState,
    buf_tx: &mut Vec<u8>,
    buf_rx: &mut Vec<u8>,
) -> AnyResult<()> {
    match proto {
        ProtoState::Start(start) => {
            let handshake = mw_proto_host::ConnectHandshake {
                client_version: mw_proto_host::CURRENT_VERSION,
                display_name: "Test Player!".into(),
                token: vec![],
                session_id: config.session_id,
                want_plid: None,
            };
            *proto = start.send_handshake(conn, buf_tx, handshake).await?.into();
        }
        ProtoState::HandshakeSent(awaiting) => {
            *proto = awaiting.await_handshake(buf_rx).await?.into();
        }
        ProtoState::HandshakeComplete(hscomplete) => {
        }
    }
    Ok(())
}
