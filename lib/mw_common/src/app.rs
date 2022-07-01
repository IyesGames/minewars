//! Stuff for the Bevy client app, that also needs to be accessible from the proprietary plugin.

use iyesengine::prelude::*;

use crate::game::TileKind;
use crate::plid::PlayerId;

/// State type: If we are in-game, where is the gameplay data coming from?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum StreamSource {
    /// No active data source
    Disconnected,
    /// Network server
    Connected,
    /// We are the server (LAN)
    Host,
    /// Hosting for ourselves only (offline)
    Local,
    /// Replay file
    File,
}

/// State type: If we are in-game, which mode are we in?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum GameMode {
    /// Not in game
    None,
    /// Playing multiplayer
    /// (see also [`StreamSource`])
    Multiplayer,
    /// Watching (replays or live)
    /// (see also [`StreamSource`])
    Spectator,
    /// Singleplayer Minesweeper (classic) mode
    Singleplayer,
    /// Tutorial
    Tutorial,
    /// Developer Mode
    Dev,
}

/// State type: Which "screen" is the app in?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AppGlobalState {
    /// Initial loading screen at startup
    AssetsLoading,
    /// Splash with the IyesGames logo
    SplashIyes,
    /// Splash with the Bevy logo
    SplashBevy,
    /// Main Menu
    MainMenu,
    /// Menu/UI for entering into a given game mode
    /// (see also [`GameMode`])
    GameLobby,
    /// The loading screen before gameplay starts
    /// (see also [`GameMode`])
    GameLoading,
    /// Gameplay
    /// (see also [`GameMode`])
    InGame,
}

impl Component for TileKind {
    type Storage = bevy::ecs::component::TableStorage;
}

/// The PlayerId that the user is playing as
pub struct ActivePlid(pub PlayerId);
