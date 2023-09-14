use std::net::SocketAddr;

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
        for addr in config.server.listen_players.iter() {
            let jh = tokio::spawn(host_listener(config.clone(), listener_kill_tx.subscribe(), *addr));
            jhs_listeners.push(jh);
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
    addr: SocketAddr,
) {
    info!("Listening for incoming player connections on: {}", addr);

    loop {
        tokio::select! {
            Ok(()) = kill_rx.recv() => {
                break;
            }
        }
    }
}
