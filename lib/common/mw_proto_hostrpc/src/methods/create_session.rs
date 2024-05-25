use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::{RpcMethod, RpcMethodName};

/// RPC method: create a new game session on the host
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct CreateSession {
    /// If set, require a specific session id value to be used
    pub session_id: Option<u64>,
    /// How many plids will the game contain + any configuration for each one
    pub plids: Vec<PlidConfig>,
    /// Where to get the map data from?
    pub map_source: MapSource,
}

impl RpcMethod for CreateSession {
    const NAME: RpcMethodName = RpcMethodName::CreateSession;
    type Error = CreateSessionError;
}

#[derive(Debug, Error)]
#[derive(Serialize, Deserialize)]
pub enum CreateSessionError {
    #[error("Session ID in use.")]
    SessionIdInUse,
    // ... TODO
}

/// Properties of a plid in a session
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct PlidConfig {
    /// How many clients control the same plid?
    /// Typically 1, but can be more, for game modes like Duos.
    pub n_clients: u8,
}

/// What map data to use for a session?
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum MapSource {
    /// Generate a blank map (classic full grid)
    Flat,
    /// Generate a new random MineWars map
    Procedural {
        /// If non-zero, use a specific seed. If zero, use random seed.
        seed: u64,
    },
    /// Load the map from a MineWars replay/scenario file
    File {
        /// Relative path to file; up to the host server how to interpret it
        path: PathBuf,
    },
    /// Receive the map as a RPC payload (payload must be a MineWars file)
    Payload,
}
