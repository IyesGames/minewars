use crate::{prelude::*, plid::{PlayerId, Plids}};

/// Abstract interface through which the Game communicates with the Host
///
/// The "Host" is the implementation of the game session. For example:
/// a networked MineWars session (the proprietary "tokio host") or an
/// offline/debug session in Bevy ("bevy host"). Each one should
/// implement this trait using its respective timers, events, etc.
pub trait Host<G: Game>: Sized {
    /// Notify the Host about something that happened in the game world
    fn msg(&mut self, plids: Plids, event: G::OutEvent);
    /// Request an action to occur at a specific future time
    fn sched(&mut self, time: Instant, event: G::SchedEvent);
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
pub trait Game: Sized {
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
    type SchedEvent: Eq + Hash + Send + Sync + 'static;

    /// For things that are triggered by player input
    ///
    /// Host code should call `Game::input_action`, passing an appropriate
    /// value, whenever the player wishes to perform an action in the game.
    type InputAction: Clone + Send + Sync + 'static;

    /// Data passed at initialization to set up the game.
    type InitData: Send + Sync + 'static;

    /// Called once, at the start, before anything else
    fn init<H: Host<Self>>(&mut self, host: &mut H, initdata: Self::InitData);

    /// Trigger a timer-driven event in the game
    fn unsched<H: Host<Self>>(&mut self, host: &mut H, event: Self::SchedEvent);

    /// Process a player input
    fn input<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, action: Self::InputAction);
}

