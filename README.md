# MineWars Game

MineWars is a competitive multiplayer real-time strategy game based on the
classic Minesweeper.

The game has been in design and development on-and-off for a few years, and has
gone through several design iterations, prototypes, and rewrites during that
time. Those old versions are not publicly available.

This repo hosts the current version of the game, that is being developed in
an effort to finally make a public alpha release of the game. :)

The current state is WIP and this version of the game is not playable yet.
Stay tuned for updates.

## Documentation

The main body of documentation is a `mdbook`, sources in `doc/`.
It contains detailed game design documents and specifications.

## License

The majority of the game code is Free, Open-Source Software. Everything
in this repo is dual-licensed as MIT/Apache-2.

The game also has proprietary parts. Those are in a separate private repo. Any
official builds of the game (as distributed to players via official channels)
are proprietary.

You can compile an open-source build of the game using just this repo, without
the proprietary parts. It will have limited functionality (notably, you will not
be able to play the main multiplayer game mode). [See here for more
info.](./doc/src/foss.md)

```sh
cargo run --features dev
```

(the `dev` feature enables extra functionality not present in release builds)

The [minewars-proprietary-stub](https://github.com/IyesGames/minewars-proprietary-stub)
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

 - Top-level: the main game client (Bevy)
 - `bin/`: other binaries and tools
   - `mw_host`: the dedicated game server
   - `mw_hostrpc`: CLI tool for controlling and managing the Host server
   - `mw_auth`: server for ancillary multiplayer services (accounts, lobbies, matchmaking, etc.)
   - `mw_cert`: CLI tool for generating encryption certificates
   - `mw_datatool`: CLI tool for working with the MineWars data format
 - `lib/`: supporting libraries:
   - `mw_common`: common code used everywhere
   - `mw_app`: supporting client-side code between open-source and proprietary
   - `mw_dataformat`: co/dec for the format used for gameplay data
     (both replay/scenario files and over-the-wire gameplay)
   - `mw_proto_host`: protocol between the client/player and Host server
   - `mw_proto_auth`: protocol between the client/player and Auth server
   - `mw_proto_hostrpc`: RPC protocol for the Host server
   - `mw_proto_hostauth`: protocol between Host and Auth
   - `mw_game_minesweeper`: the various Minesweeper game modes (not MineWars)
 - `cfg/`: example config files and certificates for testing/development

Needless to say, everything is very WIP. Some of those crates don't have much
working code yet.

[Rust]: https://rust-lang.org
[Bevy]: https://bevyengine.org
[tokio]: https://tokio.rs
