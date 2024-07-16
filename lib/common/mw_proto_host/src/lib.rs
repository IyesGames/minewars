use serde::{Serialize, Deserialize};

use mw_common::plid::{PlayerId, Plids};
use thiserror::Error;

pub mod client;
pub mod server;

pub const CURRENT_VERSION: (u8, u8, u8, u8) = (0, 1, 0, 0);
pub const HANDSHAKE_MAGIC: &'static [u8] = b"IyesMW";

/// Upon connecting, the client must open a bi stream and send:
/// `HANDSHAKE_MAGIC` followed by this struct.
///
/// The server will respond with `Result<HandshakeSuccess, HandshakeError>`
/// and close the stream. On error, the connection will be closed.
///
/// On success, the connection stays open, and both the client and server
/// are free to open Uni/Bi streams as they desire. Every new stream opened
/// must begin with a special value to identify the purpose of the stream:
///  - `StartStreamUniClient`: for Uni streams initiated by the client
///  - `StartStreamUniServer`: for Uni streams initiated by the server
///  - `StartStreamBiClient`: for Bi streams initiated by the client
///  - `StartStreamBiServer`: for Bi streams initiated by the server
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct ConnectHandshake {
    /// Version number of client
    pub client_version: (u8, u8, u8, u8),
    /// How should we be known to other players?
    pub display_name: String,
    /// Any "session token" or password, if the server wants one
    pub token: Vec<u8>,
    /// What session id do we want to join? If None, let the server choose for us.
    pub session_id: Option<u32>,
    pub join_as: JoinAs,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct HandshakeSuccess {
    pub session_id: u32,
    pub plid: PlayerId,
    pub subplid: u8,
}

#[derive(Debug, Clone, Error)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
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

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum JoinAs {
    // Regular user. Specify what Plid you want to be.
    /// If Neutral, that means we want to join as a spectator.
    /// If None, let the server pick for us.
    Regular(Option<PlayerId>),
    // Admin: spectator who can also perform privileged actions.
    // (like kicking and banning players)
    Admin,
}

/// Upon opening a new Uni stream, the client must send this,
/// to identify the purpose of the stream.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum StartStreamUniClient {
    /// The stream will be used for game input events.
    /// Any number of serialized game input events may follow.
    InputEvents,
    /// The client confirms it is ready to start the game.
    /// No further data follows.
    ConfirmReady,
    /// The client wants to send a chat message.
    /// Exactly one `ChatSend` must follow.
    Chat(ChatSend),
    /// The client wants to transmit speech/audio.
    /// Any number of `VoipFrame`s may follow.
    /// The client should close the stream when done speaking.
    /// If the server refuses to accept Voip, it will close
    /// the stream immediately.
    Voip(VoipStartSend),
}

/// Upon opening a new Uni stream, the server must send this,
/// to identify the purpose of the stream.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum StartStreamUniServer {
    /// The server wants to start a new game.
    /// Contains the "Initialization Sequence" (see dataformat docs).
    /// The server closes the stream immediately afterwards.
    /// The client should reset all its state (except the network connection
    /// to the server) and get ready to begin a new game session.
    /// All other streams should be closed.
    InitializeSession(Vec<u8>),
    /// The stream will be used to report on the status of clients in the session.
    /// (have they connected? have they confirmed ready?)
    /// Contains an initial status for each plid+subplid.
    /// The stream may stay open and the server may send additional `PlayerStatusReport`s.
    PlayersStatus(Vec<PlayerStatus>),
    /// The gameplay officially begins.
    /// No other data follows in the stream. To be closed immediately.
    GameBegin,
    /// The server is sending a notification message for the user.
    /// Followed by one or more `Notification`s with the text that
    /// the client should display to the user.
    /// Closed immediately afterwards.
    Notification(Vec<Notification>),
    /// The stream will be used for game output events.
    /// The events will be serialized raw, with no frame header.
    /// All events assumed to be for the client's own Plid.
    /// This stream is long-lived. It can stay open for a long time
    /// and the server is free to transmit messages at any time.
    /// The client should process them immediately.
    /// Multiple such streams can be active in parallel.
    UnframedEvents,
    /// The stream will be used for framed game output events.
    /// The events will be serialized in frames (see doc on
    /// spectator dataformat), so that events for any Plid can
    /// be transmitted.
    /// This stream is long-lived. It can stay open for a long time
    /// and the server is free to transmit messages at any time.
    /// Only one such stream should be used in parallel.
    FramedEvents,
    /// The stream will be used for chat messages.
    /// Whenever other clients send chat messages, the server will relay them here.
    Chat,
    /// Another client is speaking.
    /// Their `VoipFrame`s will be relayed here.
    /// The stream will be closed by the server when they are done speaking.
    Voip(VoipStartReceive),
}

/// Upon opening a new Bi stream, the client must send this,
/// to identify the purpose of the stream.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum StartStreamBiClient {
    /// The client wants to test the connection to the server.
    /// No data must follow. The server should respond by echoing.
    Ping,
    /// The client wants to perform an admin command on the server.
    /// If the server refuses, it will close the stream immediately.
    /// (normally only allowed if the user has admin privileges).
    /// If the server accepts, the server will respond with
    /// the command's output.
    AdminCommand,
    /// The client wants to download a replay file for the game.
    /// If the server refuses, it will close the stream immediately.
    /// If the server accepts, it will respond with the data of the file
    /// and then close the stream. The client should just read to end and
    /// save the data received.
    RequestReplay,
}

/// Upon opening a new Bi stream, the server must send this,
/// to identify the purpose of the stream.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum StartStreamBiServer {
    /// The server wants to test the connection to the client.
    /// No data must follow. The client should respond by echoing.
    Ping,
}

/// See `StartStreamUniServer::PlayersStatus`.
/// May be repeated any number of times.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct PlayerStatusReport {
    pub plid: PlayerId,
    pub subplid: u8,
    pub status: PlayerStatus,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum PlayerStatus {
    /// User hasn't connected yet.
    NotConnected,
    /// User will not play.
    Gone,
    /// User connected but not yet confirmed ready.
    ConnectedNotReady,
    /// User connected and confirmed ready.
    ConnectedReady,
}

/// See `StartStreamUniServer::Notification`.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct Notification {
    pub kind: NotificationKind,
    pub message: String,
}

/// How should the client/user treat this notification?
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum NotificationKind {
    /// Miscellaneous message. To be displayed as a regular notification.
    SimpleMessage,
    /// Important message. To be displayed prominently, attention-grabbing.
    SevereMessage,
    /// Warning for the user.
    Warning,
    /// Inform the user about an issue.
    Error,
    /// "Topic" or "MOTD" message to describe the server or welcome users.
    Motd,
}

/// Who is a message (chat/voip) intended for?
/// (The server may refuse to handle some or all of these)
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub enum CommunicationRecipient {
    /// Only me: For testing, server should only send it back to the sender.
    Echo,
    /// Exactly one other user (identified by plid+subplid).
    Direct {
        plid: PlayerId,
        subplid: u8,
    },
    /// A specific set of Plids. All subplids within them will receive it.
    Plids(Plids),
    /// Other subplids playing on the same plid as the client.
    MySubplids,
    /// Everyone considered "friendly".
    AllFriendly,
    /// Everyone in the session.
    All,
}

/// See `StartStreamUniClient::Chat`.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct ChatSend {
    pub recipient: CommunicationRecipient,
    pub message: String,
}

/// See `StartStreamUniServer::Chat`.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct ChatReceive {
    pub speaking_plid: PlayerId,
    pub speaking_subplid: u8,
    pub message: String,
}

/// See `StartStreamUniClient::Voip`.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct VoipStartSend {
    pub recipient: CommunicationRecipient,
    pub first_frame: VoipFrame,
}

/// See `StartStreamUniServer::Voip`.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct VoipStartReceive {
    pub speaking_plid: PlayerId,
    pub speaking_subplid: u8,
    pub first_frame: VoipFrame,
}

/// A piece of Opus-encoded audio.
/// May be repeated any number of times before closing the stream.
/// All of them should be concatenated and treated as one long stream of audio.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(bitcode::Encode, bitcode::Decode)]
pub struct VoipFrame {
    /// Estimated duration of audio encoded, in milliseconds.
    pub duration_ms: u16,
    /// Opus-encoded audio data
    pub data: Vec<u8>,
}
