use std::net::IpAddr;

use mw_common::plid::PlayerId;
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::{RpcMethod, RpcMethodName};

/// RPC method: inform the server to expect a player to connect
///
/// Some hosts may only allow incoming player connections from clients they
/// have been told to expect, for security reasons.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct ExpectPlayer {
    /// What session id is the player supposed to join?
    pub session_id: u64,
    /// What plid will they play as?
    pub plid: PlayerId,
    /// Only allow them to connect from the specified IP address
    pub addr: Option<IpAddr>,
    /// If non-empty, require client authentication, expect this exact (DER-encoded) certificate
    pub cert: Vec<u8>,
    /// If non-empty, require this token data in the mw_proto_player handshake message
    pub token: Vec<u8>,
}

impl RpcMethod for ExpectPlayer {
    const NAME: RpcMethodName = RpcMethodName::ExpectPlayer;
    type Error = ExpectPlayerError;
}

#[derive(Debug, Error)]
#[derive(Serialize, Deserialize)]
pub enum ExpectPlayerError {
    #[error("No such session exists.")]
    UnknownSession,
    #[error("Player ID already in use.")]
    PlayerIdInUse,
    #[error("The player is banned from this host.")]
    Banned,
}

