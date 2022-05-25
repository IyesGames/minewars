//! Everything to do with driving a game session
//!
//! The types and interfaces used internally between different parts of the
//! MineWars codebase: all the implementations of game modes, client, servers,
//! replays, …, rely on this stuff.

use std::time::Instant;

use crate::grid::Pos;
use crate::plid::{Plids, PlayerId};
use crate::game::{MineKind, ProdState, ProdItem, InventoryItem};

/// Abstract interface through which the Game communicates with the Host
///
/// The "Host" is the implementation of the game session. For example:
/// a networked MineWars session (the proprietary "tokio host") or an
/// offline/debug session in Bevy ("bevy host"). Each one should
/// implement this trait using its respective timers, events, etc.
pub trait Host {
    /// Notify the Host about something that happened in the game world
    fn msg(&mut self, plids: Plids, event: GameEvent);
    /// Request an action to occur at a specific future time
    fn sched<G: Game>(&mut self, time: Instant, event: G::SchedEvent);
    /// Cancel scheduled events equal to the value given
    fn desched_all<G: Game>(&mut self, event: G::SchedEvent);
}

/// Abstract interface through which the Host communicates with the Game
///
/// The "Game" is an implementation of the mechanics for a specific game mode.
/// For example:
/// the multiplayer MineWars game (proprietary)
/// or the simplified singleplayer minesweeper mode (open-source).
pub trait Game {
    /// For things that need to be triggered on a timeout
    ///
    /// Game code calls `Host::sched`, passing a value and time instant.
    /// The host code should store the value along with a timer.
    /// When the time instant has passed, host code calls `Game::unsched`,
    /// passing the value that was stored back to the game code.
    type SchedEvent: Copy + Eq;

    /// For things that are triggered by player input
    ///
    /// Host code should call `Game::input_action`, passing an appropriate
    /// value, whenever the player wishes to perform an action in the game.
    type InputAction: Copy + Eq;

    /// Trigger a timer-driven event in the game
    fn unsched(&mut self, event: Self::SchedEvent);

    /// Process a player input
    fn input_action(&mut self, action: Self::InputAction);
}

/// Game Outputs: combined enum for all protocol messages
///
/// Anything that can be sent to a specific player:
///  - map updates
///  - inventory updates
///  - city production updates
///  - session updates
///  - statistics
///  - …
///
/// Think of this as an "intermediary representation".
///
/// It is used to represent what is going on in the game session.
///
/// On a network server, these will be buffered and encoded into actual
/// protocol messages and replay files.
///
/// In the client, these will be generated from input streams
/// (network protocol messages or replay file parser)
/// and split out into various Bevy events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    /// Set the owner of a given tile
    Owner {
        c: Pos,
        owner: PlayerId,
    },
    /// Set the digit at a given tile
    Digit {
        c: Pos,
        digit: u8,
    },
    /// Reveal or hide a mine at a given tile
    Mine {
        c: Pos,
        kind: Option<MineKind>,
    },
    /// Report activated mine at location
    MineActive {
        tile: Pos,
    },
    /// Mine/decoy exploded at tile
    Explosion {
        tile: Pos,
        kind: MineKind,
    },
    /// Road status update at tile
    Road {
        tile: Pos,
        state: ProdState,
    },
    /// City begins producing a new item
    Production {
        completed: Option<ProdItem>,
        started: ProdItem,
    },
    /// Report change to player's inventory contents (delta)
    Inventory {
        region: u8,
        item: InventoryItem,
        change: i8,
    },
    /// Player stunned; period of inactivity follows
    /// No inputs accepted from the player until respawn timeout elapses
    Stun {
        plid: PlayerId,
        /// Stun timeout in game ticks
        timeout: u16,
    },
    /// Player's stun duration ended
    StunEnd {
        plid: PlayerId,
    },
    /// Player eliminated
    GameOver {
        plid: PlayerId,
    },
}

