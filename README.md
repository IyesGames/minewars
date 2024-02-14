# MineWars Game

MineWars is a competitive multiplayer real-time strategy game with
PvP combat based on Minesweeper and Civilization-like macro gameplay.

The game has been in design and development on-and-off for a few years, and has
gone through several design iterations, prototypes, and rewrites during that
time. Those old versions are not publicly available.

This repo hosts the current version of the game, that is being developed in
an effort to finally make a public alpha release of the game. :)

The current state is WIP and this version of the game is not playable yet.
Stay tuned for updates.

## How to build and run

Note: I develop the game while working in macOS or WSL2. Everything in this
repo assumes either a Linux or macOS environment.

### Client App

To build and run the game client (macOS, non-WSL2 Linux, Windows native):

```sh
cargo run
```

If you are on Windows, there is a script you can use instead, to cross-compile
and run a Windows-native EXE from within WSL2 (uses the `-gnu` Rust Toolchain):

```sh
./setup-cross-wsl2.sh # first time only
./cross-wsl2.sh
```

This gives you a game client capable of playing the classic Minesweeper game
modes (both singleplayer and multiplayer), playback of MineWars replay files,
map editor, etc. You cannot play the real MineWars game mode.

#### Release Builds

To prepare release builds with maximum optimizations and everything:

```sh
./build-release.sh {win,mac,lin}
```

(specify your OS and be sure to actually run it on the specified OS)

The final files will be in `./release/`.

### Server

If you want to play multiplayer, you also need the Host server.

```sh
cargo run --bin mw_hostsrv -- --config cfg/simple_minesweeper.cfg
```

This will build the open-source Host server and run it with the example config
that provides a single session of multiplayer Minesweeper on `localhost:13370`
with no authentication (allow any client to join or spectate). It will use the
development certificates found in `cfg/cert`.

To play on the server, launch the game client you built above, and navigate to:
Main Menu -> Play LAN -> Connect to Server. Enter:
 - Server Address: `localhost`
 - Server Port: `13370`
 - Session ID: `0`

Alternatively, press `~` to open the dev console and enter the command:

```
connect localhost:13370 0
```

### Settings

The game client stores some configs/settings in your OSs standard location
for user configuration files:
 - Windows: `C:\Users\you\AppData\Roaming\IyesGames\MineWars`
 - Mac: `/Users/you/Library/Preferences/com.iyesgames.MineWars/`
 - Linux: `/home/you/.config/minewars/`

If you want to change settings that are not (yet) changeable from in-game
UIs, you know where to look. :)

(And also if you want to delete them, so they don't litter your filesystem,
if you never want to play/develop MineWars again.)

The game does not create any other files on your system.

## Documentation

The main body of documentation is a `mdbook`, sources in `doc/`.
It contains detailed game design documents and specifications.
To render and read the book:

```sh
mdbook --open
```

Some parts (but far from all) of the source code contain Rustdoc
comments. To generate API docs, run:

```sh
cargo doc --features dev --open
```

## License

The majority of the game code is Free, Open-Source Software. All code
in this repo is dual-licensed as MIT/Apache-2.

The game also has proprietary parts. Those are in a separate private repo. Any
official builds of the game (as distributed via official channels) are
proprietary.

You can compile an open-source build of the game using just this repo, without
the proprietary parts. It will have limited functionality (notably, you will not
be able to play the main multiplayer game mode). [See here for more
info.](./doc/src/foss.md)

```sh
cargo run --features dev
```

(the `dev` feature enables extra functionality not present in release builds)

The [minewars-proprietary-shim](https://github.com/IyesGames/minewars-proprietary-shim)
repo is what makes this possible, by providing open-source stubs that will be
used instead of the real proprietary code.

Official builds of the game are also made from this repo, but with the real
proprietary code instead of the stubs. This repo here is not just a code dump,
this is where the actual development happens. If you make contributions to the
open-source code here, your contributions will be included (and you will be
credited for them) in official builds of the game.

## Source Code

The game is implemented in the [Rust] programming language. The client
is made with the [Bevy] game engine, but also uses [tokio] in the background
for networking. The servers do not use [Bevy], and are pure [tokio].

Given that this is a multiplayer game, and there are many parts to it
(game client, servers, tooling, etc.), this repo contains many Rust crates:

 - Top-level (`minewars`): the main game client binary
 - `bin/`: other binaries and tools
   - `mw_hostsrv`: the dedicated game server
   - `mw_hostrpc`: CLI tool for controlling and managing the Host server
   - `mw_authsrv`: server for non-gameplay multiplayer services (accounts, lobbies, matchmaking, etc.)
   - `mw_certgen`: CLI tool for generating encryption certificates
   - `mw_datatool`: CLI tool for working with the MineWars data format
 - `lib/`: supporting libraries:
   - `mw_common`: common code for everything
   - `mw_app`: the open-source part of the client
   - `mw_host`: the open-source part of the Host server
   - `mw_auth`: the open-source part of the Auth server
   - `mw_dataformat`: co/dec for the format used for gameplay data
     (both replay/scenario files and over-the-wire gameplay)
   - `mw_proto_host`: protocol between the client/player and Host server
   - `mw_proto_auth`: protocol between the client/player and Auth server
   - `mw_proto_hostrpc`: RPC protocol for the Host server
   - `mw_proto_hostauth`: protocol between Host and Auth
   - `mw_game_minesweeper`: the various Minesweeper game modes (not MineWars)
 - `cfg/`: example config files and certificates for testing/development

Crates from the Proprietary Repo (not publicly available, just stubbed):
 - `mw_app_proprietary`: proprietary parts of the client app
 - `mw_host_proprietary`: proprietary parts of the Host server
 - `mw_auth_proprietary`: proprietary parts of the Auth server
 - `mw_game_minewars`: the full-fledged MineWars game mode

The top-level crate (`minewars`) just combines `mw_app` and (optionally)
`mw_app_proprietary` (not in this repo) to create a single executable binary.
All of the actual code for the client app lives in those crates.

Similarly:
 - `mw_hostsrv` creates a binary out of `mw_host` and (optionally) `mw_host_proprietary`
 - `mw_authsrv` creates a binary out of `mw_auth` and (optionally) `mw_auth_proprietary`

Needless to say, everything is very WIP. Some of those crates don't have much
working code yet.

[Rust]: https://rust-lang.org
[Bevy]: https://bevyengine.org
[tokio]: https://tokio.rs

## Assets

This repo contains some basic assets needed to run MineWars.

 - `assets/font/Sansation-*.ttf`: SIL Open Font License
 - `assets/font/Ubuntu-*.ttf`: Ubuntu Font License
 - `assets/splash/bevy.png`: Bevy logo used with permission from Bevy Project
 - `assets/splash/iyes*.png`: IyesGames Logo; contact for permission to use outside of MineWars
 - `assets/gfx2d/*`: CC-BY-SA 4.0
 - `assets/gfx3d/*`: CC-BY-SA 4.0
 - `assets/locale/*`: CC-BY-SA 4.0
 - `assets/ui/*`: CC-BY-SA 4.0
 - `assets/audio/*`: CC-BY-SA 4.0

The `assets-src` folder contains the source files (Blender, Inkscape, etc.) used to generate
the various assets that are not sourced externally. There are also some scripts to process
them if needed.

The official builds of the game contain extra/different assets, that are proprietary.
If you have an official copy of the game, feel free to use those assets for private use
with open-source builds of the game. They should be compatible. Just copy them into the
`assets` folder.
