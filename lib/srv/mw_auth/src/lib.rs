#![allow(unused_variables)]

pub mod prelude {
    pub use mw_common::prelude::*;
    pub use crate::config::Config;
    pub use tracing::{error, warn, info, debug, trace};
    pub use tokio::sync::{Mutex, RwLock, Notify};
    pub use tokio::time::{Duration, Instant};
    pub use clap;
    pub use tracing;
    pub use tracing_subscriber;
}

use crate::prelude::*;

pub mod cli;
pub mod config;

pub async fn load_config(path: &Path, cli: &cli::Args) -> AnyResult<Arc<Config>> {
    let config_bytes = tokio::fs::read(path).await
        .context("Could not read config file")?;
    let config_str = std::str::from_utf8(&config_bytes)
        .context("Config file is not UTF-8")?;
    let mut config: Config = toml::from_str(config_str)
        .context("Error in config file")?;
    config.apply_cli(cli);
    Ok(Arc::new(config))
}

/// This is the stuff that should be set up once and preserved if the config is reloaded.
pub struct PersistSoftReset {
}

impl PersistSoftReset {
    pub async fn do_soft_reset(&self) {
    }
}

/// If proprietary is enabled, this is the data it needs to have access to.
/// This struct is to be passed into its setup function.
pub struct ForProprietary {
}

pub async fn init(_rt: ManagedRuntime) -> PersistSoftReset {
    PersistSoftReset {
    }
}

pub async fn setup_with_config(
    rt: ManagedRuntime,
    persist: &PersistSoftReset,
    softreset: CancellationToken,
    config: Arc<Config>,
) -> ForProprietary {
    // TODO
    ForProprietary {
    }
}
