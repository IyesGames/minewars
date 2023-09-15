pub mod prelude {
    pub use mw_common::prelude::*;
    pub use crate::config::Config;
    pub use tracing::{error, warn, info, debug, trace};
}

use clap::Parser;

use crate::prelude::*;

mod cli;
mod config;
mod hostauth;
mod rpc;
mod server;

fn main() {
    let args = cli::Args::parse();

    let config_bytes = std::fs::read(&args.config)
        .expect("Could not read config file");
    let config_str = std::str::from_utf8(&config_bytes)
        .expect("Config file is not UTF-8");
    let mut config: Config = toml::from_str(config_str)
        .expect("Error in config file");

    config.apply_cli(&args);

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
             use std::sync::atomic::*;
             static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
             let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
             format!("minewars-host-worker-{}", id)
        })
        .build()
        .expect("Cannot create tokio runtime!");

    rt.block_on(async_main(Arc::new(config)));
}

async fn async_main(config: Arc<Config>) {
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);
    let (reload_tx, mut reload_rx) = tokio::sync::broadcast::channel::<Arc<Config>>(1);

    let jh_server = tokio::spawn(
        crate::server::host_main(
            config.clone(),
            shutdown_tx.subscribe(),
            reload_tx.subscribe(),
        )
    );

    let jh_hostauth = tokio::spawn(
        crate::hostauth::hostauth_main(
            config.clone(),
            shutdown_tx.subscribe(),
            reload_tx.subscribe(),
        )
    );

    let jh_rpc = tokio::spawn(
        crate::rpc::rpc_main(
            config.clone(),
            shutdown_tx.subscribe(),
            reload_tx.subscribe(),
        )
    );

    loop {
        tokio::select! {
            Ok(()) = shutdown_rx.recv() => {
                info!("Shutting down...");
                let _ = tokio::join! {
                    jh_server,
                    jh_hostauth,
                    jh_rpc,
                };
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl-C interrupt received! Shutting down!");
                shutdown_tx.send(()).ok();
            }
        }
    }
}
