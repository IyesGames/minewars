use serde::{Serialize, Deserialize};
use thiserror::Error;

use std::path::PathBuf;

use crate::{RpcMethod, RpcMethodName};

/// RPC method: Reload the server's configuration
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ReloadConfig {
    /// If set, load this file instead of the default
    pub path: Option<PathBuf>,
}

impl RpcMethod for ReloadConfig {
    const NAME: RpcMethodName = RpcMethodName::ReloadConfig;
    type Error = ReloadConfigError;
}

#[derive(Debug, Error)]
#[derive(Serialize, Deserialize)]
pub enum ReloadConfigError {
    #[error("{0}")]
    Reason(String),
}
