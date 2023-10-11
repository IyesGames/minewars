use mw_common::{net::*, prelude::rustls::ServerConfig};

use crate::prelude::*;

pub async fn host_main(
    mut config: Arc<Config>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    mut reload_rx: tokio::sync::broadcast::Receiver<Arc<Config>>
) {
    info!("Host Server Initializing...");

    let mut jhs_listeners = vec![];

    loop {
        let (listener_kill_tx, _) = tokio::sync::broadcast::channel(1);

        match load_server_crypto(
            &config.server.cert,
            &config.server.key,
            !config.server.allow_players_nocert,
            &config.server.player_ca,
        ).await {
            Ok(crypto) => {
                info!("Host Server crypto (certs and keys) loaded.");
                for addr in config.server.listen_players.iter() {
                    let jh = tokio::spawn(host_listener(config.clone(), listener_kill_tx.subscribe(), crypto.clone(), *addr));
                    jhs_listeners.push(jh);
                }
            }
            Err(e) => {
                error!("Host Server crypto (certs/keys) failed to load: {}", e);
            }
        }

        tokio::select! {
            Ok(()) = shutdown_rx.recv() => {
                listener_kill_tx.send(()).ok();
                for jh in jhs_listeners.drain(..) {
                    jh.await.ok();
                }
                break;
            }
            Ok(newconfig) = reload_rx.recv() => {
                config = newconfig;
                // stop all existing listeners and create new ones next loop
                listener_kill_tx.send(()).ok();
                // wait for the old ones to stop
                for jh in jhs_listeners.drain(..) {
                    jh.await.ok();
                }
            }
        }
    }
}

async fn host_listener(
    config: Arc<Config>,
    mut kill_rx: tokio::sync::broadcast::Receiver<()>,
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
            Ok(()) = kill_rx.recv() => {
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
