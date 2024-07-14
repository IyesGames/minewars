use mw_common::net::*;
use mw_proto_host::server::ProtoState;
use quinn::crypto::rustls::QuicServerConfig;

use crate::prelude::*;

pub async fn host_main(
    config: Arc<Config>,
    rt: ManagedRuntime,
    softreset: CancellationToken,
) {
    info!("Host Server Initializing...");
    let server_settings = ServerSettings {
        server_certs: config.server.cert.clone(),
        server_key: config.server.key.clone(),
        client_ca: if config.server.allow_players_nocert {
            vec![]
        } else {
            config.server.player_ca.clone()
        },
    };
    match load_server_crypto(
        &server_settings,
    ).await {
        Ok(crypto) => {
            info!("Host Server crypto (certs and keys) loaded.");
            for addr in config.server.listen_players.iter() {
                rt.spawn(
                    host_listener(
                        config.clone(),
                        rt.clone(),
                        softreset.clone(),
                        crypto.clone(),
                        *addr,
                    )
                );
            }
        }
        Err(e) => {
            error!("Host Server crypto (certs/keys) failed to load: {}", e);
        }
    }
}

async fn host_listener(
    config: Arc<Config>,
    rt: ManagedRuntime,
    softreset: CancellationToken,
    crypto: Arc<QuicServerConfig>,
    addr: SocketAddr,
) {
    let endpoint = match setup_quic(addr, Some(crypto), None) {
        Ok(endpoint) => endpoint,
        Err(e) => {
            error!("Failed to create QUIC Endpoint: {}", e);
            return;
        }
    };

    info!("Listening for incoming player connections on: {}", addr);

    loop {
        tokio::select! {
            _ = softreset.cancelled() => {
                break;
            }
            _ = rt.listen_shutdown() => {
                break;
            }
            incoming = endpoint.accept() => {
                match incoming {
                    Some(incoming) => {
                        let rt = rt.clone();
                        let config = config.clone();
                        tokio::spawn(async {
                            if let Err(e) = player_handle_connection(rt, config, incoming).await {
                                error!("Player connection error: {}", e);
                            }
                        });
                    }
                    None => {
                        error!("Player endpoint for {} closed!", addr);
                        break;
                    }
                }
            }
        }
    }
}

async fn player_handle_connection(
    rt: ManagedRuntime,
    config: Arc<Config>,
    incoming: quinn::Incoming,
) -> AnyResult<()> {
    let addr_remote = incoming.remote_address();
    if !check_list(config.server.ip_control, config.server.ip_list.temporary_todo_unwrap(), &addr_remote.ip()) {
        info!("Ignoring incoming Player connection from banned IP: {}", addr_remote);
        incoming.ignore();
        return Ok(());
    }
    // TODO: player expectation

    info!("Incoming Player connection from: {}", addr_remote);
    let conn = incoming.await?;

    info!("Player connected from: {}", addr_remote);

    let mut buf_tx = Vec::with_capacity(512);
    let mut buf_rx = vec![0; 512];
    let mut proto = ProtoState::new();

    loop {
        tokio::select! {
            _ = rt.listen_shutdown() => {
                break;
            }
            e = conn.closed() => {
                info!("Player connection {} closed: {}", addr_remote, e);
                break;
            }
            r = drive_player_session(&conn, &mut proto, &mut buf_tx, &mut buf_rx) => {
                if let Err(e) = r {
                    error!("Player<->Host Protocol Error: {:#}", e);
                    break;
                }
            }
        }
    }

    info!("Player {} disconnected.", addr_remote);

    Ok(())
}

async fn drive_player_session(
    conn: &quinn::Connection,
    proto: &mut ProtoState,
    buf_tx: &mut Vec<u8>,
    buf_rx: &mut Vec<u8>,
) -> AnyResult<()> {
    match proto {
        ProtoState::AwaitingHandshake(awaiting) => {
            let (responding, handshake) = awaiting.await_handshake(conn, buf_rx).await?;
            *proto = responding.into();
        }
        ProtoState::HandshakeResponding(responding) => {
            let response = Err(mw_proto_host::HandshakeError::Full);
            *proto = responding.respond_handshake(buf_tx, &response).await?.into();
        }
        ProtoState::HandshakeComplete(hscomplete) => {
        }
    }
    Ok(())
}
