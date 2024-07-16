use crate::{prelude::*, plid::{PlayerId, Plids}};

/// Abstract interface through which the Game communicates with the Host
///
/// The "Host" is the implementation of the game session. For example:
/// a networked MineWars session (the proprietary "tokio host") or an
/// offline/debug session in Bevy ("bevy host"). Each one should
/// implement this trait using its respective timers, events, etc.
pub trait Host<G: Game>: Sized + Send + Sync + 'static {
    /// Notify the Host about something that happened in the game world
    fn msg(&mut self, plids: Plids, event: G::OutEvent);
    /// Request an action to occur at a specific future time
    fn sched(&mut self, time: std::time::Instant, event: G::SchedEvent);
    /// Cancel scheduled events equal to the value given
    fn desched_all(&mut self, event: G::SchedEvent);
    fn game_over(&mut self);
}

/// Abstract interface through which the Host communicates with the Game
///
/// The "Game" is an implementation of the mechanics for a specific game mode.
/// For example:
/// the multiplayer MineWars game (proprietary)
/// or the simplified singleplayer minesweeper mode (open-source).
pub trait Game: Sized + Send + Sync + 'static {
    /// Anything that happened that needs to be communicated to a player
    ///
    /// When the Host calls either `Game::input_action` or `Game::unsched`
    /// to drive the Game, it can call `Host::msg` to send output events.
    /// They will be broadcast to all player ids selected with `Plids`.
    type OutEvent: Clone + Send + Sync + 'static;

    /// For things that need to be triggered on a timeout
    ///
    /// Game code calls `Host::sched`, passing a value and time instant.
    /// The host code should store the value along with a timer.
    /// When the time instant has passed, host code calls `Game::unsched`,
    /// passing the value that was stored back to the game code.
    type SchedEvent: Clone + Eq + Hash + Send + Sync + 'static;

    /// For things that are triggered by player input
    ///
    /// Host code should call `Game::input_action`, passing an appropriate
    /// value, whenever the player wishes to perform an action in the game.
    type InputAction: Clone + Send + Sync + 'static;

    /// Data passed at initialization to set up the game.
    type InitData: Send + Sync + 'static;

    /// Called once, at the start, before anything else
    fn init<H: Host<Self>>(&mut self, host: &mut H, initdata: Box<Self::InitData>);

    /// Trigger a timer-driven event in the game
    fn unsched<H: Host<Self>>(&mut self, host: &mut H, event: Self::SchedEvent);

    /// Process a player input
    fn input<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, action: Self::InputAction);

    /// Query the game for data / status updates to put in an unreliable datagram
    fn unreliable<H: Host<Self>>(&mut self, _host: &mut H) {}

    fn needs_maintain(&self) -> bool {
        false
    }

    fn maintain(&mut self) {}
}

/// Abstract interface for writing multiple player streams at once.
///
/// We use this to encode game messages on the server side into all the appropriate
/// player streams.
pub trait PlidsMuxWriter {
    /// Append some bytes to each of the buffers as indicated by Plids.
    fn append_bytes(
        &mut self,
        plids: Plids,
        bytes: &[u8]
    ) -> std::io::Result<()>;
}

/// Abstract interface for outputting game events when decoding a stream.
///
/// When a protocol stream is decoded, this trait will help handle each event
/// on the client side.
pub trait EventEmitter<G: Game> {
    type EmitError: std::error::Error;

    fn emit_event(&mut self, plid: PlayerId, event: G::OutEvent) -> Result<(), Self::EmitError>;
}

/// Abstract interface for serializing a Game's OutEvents into protocol streams
///
/// Used for games hosted on a network server, where the event stream needs to
/// be sent over a network transport protocol.
pub trait Encoder<G: Game, W: PlidsMuxWriter>: Sized {
    type EncodeError: std::error::Error;

    /// Takes a queue of game events with associated destination Plids, generates
    /// a binary stream to be sent to each PlayerId.
    ///
    /// Implementations are free to mangle and mutate the queue in-place as they see fit
    /// (typically for optimization purposes). It should be cleared before returning.
    fn encode(
        &mut self,
        writer: &mut W,
        event_q: &mut Vec<(Plids, G::OutEvent)>
    ) -> Result<(), Self::EncodeError>;
}

/// Abstract interface for deserializing a Game's OutEvents from protocol streams
///
/// Used for game clients playing on a network server, where the event stream needs
/// to be received over a network transport protocol.
pub trait Decoder<G: Game, E: EventEmitter<G>>: Sized {
    type DecodeError: std::error::Error;

    /// Takes a stream of bytes and decodes it into game events.
    ///
    /// The emitter is used to output each decoded event. Using a generic interface
    /// like this allows the game client to control what to do with each event.
    fn decode(&mut self, emitter: &mut E, bytes: &[u8]) -> Result<(), Self::DecodeError>;
}
