use serde::{Serialize, Deserialize};

use mw_common::plid::PlayerId;
use thiserror::Error;

/// The first packet sent by a game client, to identify itself to the host
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ConnectHandshake {
    /// How should we be known to other players?
    pub display_name: String,
    /// Any "session token", if the server wants one
    pub token: Vec<u8>,
    /// What session id do we want to join? If None, let the server choose for us.
    pub session_id: Option<u64>,
    /// What PlayerId do we want to play as?
    /// If Neutral, that means we want to join as a spectator.
    /// If None, let the server pick for us.
    pub want_plid: Option<PlayerId>,
    /// If enabled, disable all social interaction with other players.
    /// Their names/profiles will be anonymized and communications will be disabled.
    pub antisocial_mode: bool,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct HandshakeSuccess {
    pub plid: PlayerId,
}

#[derive(Debug, Clone, Error)]
#[derive(Serialize, Deserialize)]
pub enum HandshakeError {
    #[error("Your request is invalid.")]
    Invalid,
    #[error("You requested something unsupported or disabled.")]
    Unsupported,
    #[error("You are not allowed to join the session as you requested.")]
    Forbidden,
    #[error("You are banned from this server.")]
    Banned,
    #[error("Session full. There is no space for you.")]
    Full,
}
