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
