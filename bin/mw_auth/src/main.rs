pub mod prelude {
    pub use mw_common::prelude::*;
    pub use crate::config::Config;
    pub use tracing::{error, warn, info, debug, trace};
}

use clap::Parser;

use crate::prelude::*;

mod cli;
mod config;

fn main() {
    let args = cli::Args::parse();

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
             format!("minewars-auth-worker-{}", id)
        })
        .build()
        .expect("Cannot create tokio runtime!");

    rt.block_on(async_main(Arc::new(config)));
}

async fn load_config(path: &Path) -> AnyResult<Config> {
    let config_bytes = tokio::fs::read(path).await
        .context("Could not read config file")?;
    let config_str = std::str::from_utf8(&config_bytes)
        .context("Config file is not UTF-8")?;
    let config: Config = toml::from_str(config_str)
        .context("Error in config file")?;
    Ok(config)
}

async fn async_main(config: Arc<Config>) {
    let rt = ManagedRuntime::new();

    loop {
        let softreset = rt.child_token();

        let mut config = match load_config(&args.config).await {
            Ok(mut config) => {
                config.apply_cli(&args);
                Arc::new(config)
            }
            Err(e) => {
                error!("Error loading config file: {:#}", e);
                if rt.has_tasks() {
                    let sec_retry = 15;
                    info!("Server has active tasks. Will not shut down; retrying config load in {} seconds.", sec_retry);
                    tokio::time::sleep(Duration::from_secs(sec_retry)).await;
                    continue;
                } else {
                    break;
                }
            }
        };

        // TODO

        tokio::select! {
            _ = softreset.cancelled() => {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            _ = rt.listen_shutdown() => {
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl-C interrupt received!");
                break;
            }
        }
    }

    info!("Shutting down...");
    rt.trigger_shutdown();
    rt.wait_shutdown().await;
    info!("Shutdown complete.");
}
