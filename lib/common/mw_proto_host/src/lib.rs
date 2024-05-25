use serde::{Serialize, Deserialize};

use mw_common::plid::PlayerId;
use thiserror::Error;

pub mod client;
pub mod server;

pub const CURRENT_VERSION: (u8, u8, u8, u8) = (0, 1, 0, 0);
pub const HANDSHAKE_MAGIC: &'static [u8] = b"IyesMW";

/// The first packet sent by a game client, to identify itself to the host
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ConnectHandshake {
    /// Version number of client
    pub client_version: (u8, u8, u8, u8),
    /// How should we be known to other players?
    pub display_name: String,
    /// Any "session token" or password, if the server wants one
    pub token: Vec<u8>,
    /// What session id do we want to join? If None, let the server choose for us.
    pub session_id: Option<u32>,
    /// What PlayerId do we want to play as?
    /// If Neutral, that means we want to join as a spectator.
    /// If None, let the server pick for us.
    pub want_plid: Option<PlayerId>,
}

/// What stage is the session in?
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum ProtoMode {
    /// Game not started yet, waiting for players.
    WaitPlayers,
    /// Minesweeper game mode.
    MinesweeperGame,
    /// MineWars pre-game (pick/ban) stage.
    MinewarsPreGame,
    /// MineWars main gameplay.
    MinewarsGame,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct HandshakeSuccess {
    pub session_id: u32,
    pub plid: PlayerId,
    pub mode: ProtoMode,
}

#[derive(Debug, Clone, Error)]
#[derive(Serialize, Deserialize)]
pub enum HandshakeError {
    #[error("Your request is invalid.")]
    Invalid,
    #[error("Client version too old. Please update.")]
    VersionTooOld,
    #[error("Client version too new. Server incompatible.")]
    VersionTooNew,
    #[error("You requested something unsupported or disabled.")]
    Unsupported,
    #[error("You are not allowed to join the session as you requested.")]
    Forbidden,
    #[error("You are banned from this server.")]
    Banned,
    #[error("Session full. There is no space for you.")]
    Full,
}
