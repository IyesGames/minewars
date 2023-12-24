use crate::prelude::*;

pub async fn hostauth_main(
    config: Arc<Config>,
    rt: ManagedRuntime,
    softreset: CancellationToken,
) {
    if config.hostauth.enable {
        info!("HostAuth Client initializing...");
        rt.spawn(
            hostauth_client(
                config.clone(),
                rt.clone(),
                softreset.clone(),
                config.hostauth.server,
            )
        );
    } else {
        info!("HostAuth disabled in config.");
    }
}

async fn hostauth_client(
    config: Arc<Config>,
    rt: ManagedRuntime,
    softreset: CancellationToken,
    addr: SocketAddr,
) {
    info!("Connecting to HostAuth Server: {}", addr);

    loop {
        tokio::select! {
            _ = rt.listen_shutdown() => {
                break;
            }
        }
    }
}
