# MineWars Game

MineWars is a competitive multiplayer real-time strategy game with
PvP combat based on Minesweeper and Civilization-like macro gameplay.

The game has been in design and development on-and-off for a few years, and has
gone through several design iterations, prototypes, and rewrites during that
time. Those old versions are not publicly available.

The current version of the game is currently in-development, in an effort
to finally make a public alpha release. :)

The current state is WIP and this version of the game is not playable yet.
Stay tuned for updates.

The full game is proprietary and source code is not publicly available. The
client app (made using the Bevy game engine) is partly-open-source, for the
benefit of the Bevy game dev community. The open-source parts are in this repo.

## How to build and run

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
modes offline, playback of MineWars replay files, map editor, etc. You cannot
play the real MineWars game mode.

#### Release Builds

To prepare release builds with maximum optimizations and everything:

```sh
./build-release.sh {win,mac,lin}
```

(specify your OS and be sure to actually run it on the specified OS)

The final files will be in `./release/`.

### Server and Multiplayer

Networked multiplayer is not part of the FOSS repo. Sorry.

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

All code in this repo is dual-licensed as MIT/Apache-2.

The game also has proprietary parts. Those are in a separate private repo. Any
official builds of the game (as distributed via official channels) are
proprietary.

You can compile an open-source build of the game using just this repo, without
the proprietary parts. It will have limited functionality (notably, you will not
be able to play the actual MineWars game mode). [See here for more
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

This repo contains many Rust crates:

 - Top-level (`minewars`): the main game client binary (Desktop Platforms)
 - `mobile/`: the main game client binary (Mobile Platforms)
 - `bin/`: other binaries and tools
   - `mw_certgen`: CLI tool for generating encryption certificates
   - `mw_datatool`: CLI tool for working with the MineWars data format
 - `lib/app`: library crates that form the Bevy-based client app:
   - `mw_engine`: building blocks and bespoke tech
   - `mw_app_core`: common framework for all of the following crates
   - `mw_app`: implementation of the functionality of the game client
     (sans UI and graphics)
   - `mw_app_gfx2d`: 2D graphics
   - `mw_app_gfx3d`: 3D graphics
   - `mw_ui_common`: UI building blocks
   - `mw_ui_desktop`: the desktop UI
   - `mw_ui_mobile`: the mobile UI
   - `mw_platform_windows`: Windows-specific code
   - `mw_platform_macos`: MacOS-specific code
   - `mw_platform_linux`: Linux-specific code
   - `mw_platform_android`: Android-specific code
   - `mw_platform_ios`: iOS-specific code
   - `mw_app_game_minesweeper`: App integration for `mw_game_minesweeper`
 - `lib/common`: library crates used by both client and server:
   - `mw_common`: common code for everything
   - `mw_game_minesweeper`: minimal open-source Minesweeper game mode
   - `mw_dataformat`: co/dec for the format used for gameplay data
     (in both replay/scenario files and over-the-wire gameplay protocol)

All of the actual code for the client app lives in library crates.

The top-level crate (`minewars_foss`) just creates a single executable
binary by combining them all.

The `mobile` crate is very similar; it creates the mobile apps for
iOS/Android by including all of the above + extra mobile-specific stuff.

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

The `assets-src` folder contains the source files (Blender, Inkscape,
etc.) used to generate the various assets that are not sourced
externally. There are also some scripts to process them if needed.

The official builds of the game contain extra/different assets, that
are proprietary. If you have an official copy of the game, feel free to
use those assets for private use with open-source builds of the game. They
should be compatible. Just copy them into the `assets` folder.
