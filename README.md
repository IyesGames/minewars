# MineWars Game Client

This is the client app of MineWars, made using Bevy.

NOTE: this is still a newly-created repo, where I am migrating my existing work
on the game, that was done in private, with the intention of open-sourcing
much of it. This is still WIP. Don't use yet! This note will be removed from
the README when this repo is fully set up. Future development of the game
will continue from here.

## Repo Structure

 - `doc`:
   `mdbook` with the game design and some technical documentation
 - `lib/mw_dataformat`:
   standalone library for working with the MineWars data stream format
 - `lib/mw_common`:
   shared between the open-source and proprietary parts of the project

## Proprietary Features

My goal is to help the Bevy community while I make my game, by providing an
open-source codebase that others can learn and benefit from.

I intend to open-source all the Bevy-centric parts of the code, but I want
to keep some key components of the game proprietary.

The intention is: when you build this repo with default cargo features,
you should get a fully-functional MineWars **viewer**, capable of:
 - Playing back replay files
 - Spectating live games

Note that this effectively includes almost all the Bevy code of the game,
and most of the technology that people in the Bevy community have wanted to
learn from my project. UI, rendering, input, etc … are all included. So
is some of the networking (for spectating).

However, the open-source build does *not* provide the following features:
 - Hosting games (no server, LAN, or Playground modes)
 - Multiplayer (connecting to a server and playing a game)

(basically, you cannot actually play the game)

These are enabled with a `proprietary` cargo feature, which pulls in an
external dependency where they are implemented, which is not publicly
available.
