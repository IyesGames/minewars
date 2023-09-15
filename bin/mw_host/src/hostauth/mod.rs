use crate::prelude::*;

pub async fn hostauth_main(
    mut config: Arc<Config>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    mut reload_rx: tokio::sync::broadcast::Receiver<Arc<Config>>
) {
    info!("HostAuth Client initializing...");

    let mut jh_client = None;

    loop {
        let (client_kill_tx, _) = tokio::sync::broadcast::channel(1);

        if config.hostauth.enable {
            jh_client = Some(tokio::spawn(
                hostauth_client(config.clone(), client_kill_tx.subscribe(), config.hostauth.server)
            ));
        } else {
            info!("HostAuth disabled in config.");
        }

        tokio::select! {
            Ok(()) = shutdown_rx.recv() => {
                client_kill_tx.send(()).ok();
                if let Some(jh) = jh_client.take() {
                    jh.await.ok();
                }
                break;
            }
            Ok(newconfig) = reload_rx.recv() => {
                config = newconfig;
                // stop existing client and create new one next loop
                client_kill_tx.send(()).ok();
                // wait for the old one to stop
                if let Some(jh) = jh_client.take() {
                    jh.await.ok();
                }
            }
        }
    }
}

async fn hostauth_client(
    config: Arc<Config>,
    mut kill_rx: tokio::sync::broadcast::Receiver<()>,
    addr: SocketAddr,
) {
    info!("Connecting to HostAuth Server: {}", addr);

    loop {
        tokio::select! {
            Ok(()) = kill_rx.recv() => {
                break;
            }
        }
    }
}
