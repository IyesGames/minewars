use mw_common::{net::*, prelude::rustls::ServerConfig};

use crate::prelude::*;

pub async fn host_main(
    config: Arc<Config>,
    rt: ManagedRuntime,
    softreset: CancellationToken,
) {
    info!("Host Server Initializing...");
    match load_server_crypto(
        &config.server.cert,
        &config.server.key,
        !config.server.allow_players_nocert,
        &config.server.player_ca,
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
    crypto: Arc<ServerConfig>,
    addr: SocketAddr,
) {
    let endpoint = match setup_quic_server(crypto, addr) {
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
            connecting = endpoint.accept() => {
                match connecting {
                    Some(connecting) => {
                        let config = config.clone();
                        tokio::spawn(async {
                            if let Err(e) = player_handle_connection(config, connecting).await {
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
    config: Arc<Config>,
    connecting: quinn::Connecting,
) -> AnyResult<()> {
    let addr_remote = connecting.remote_address();
    if !check_list(config.server.ip_control, config.server.ip_list.temporary_todo_unwrap(), &addr_remote.ip()) {
        info!("Ignoring incoming Player connection from banned IP: {}", addr_remote);
        return Ok(());
    }
    // TODO: player expectation

    info!("Incoming Player connection from: {}", addr_remote);
    let conn = connecting.await?;
    info!("Player connected from: {}", addr_remote);
    Ok(())
}
