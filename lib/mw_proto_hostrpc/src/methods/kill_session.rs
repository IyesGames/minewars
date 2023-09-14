use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::{RpcMethod, RpcMethodName};

/// RPC method: Forcefully terminate a game session
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct KillSession {
    /// The session id to terminate
    pub session_id: u64,
}

impl RpcMethod for KillSession {
    const NAME: RpcMethodName = RpcMethodName::KillSession;
    type Error = KillSessionError;
}

#[derive(Debug, Error)]
#[derive(Serialize, Deserialize)]
pub enum KillSessionError {
    #[error("No such session exists.")]
    UnknownSession,
}
