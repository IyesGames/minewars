# Game Longevity and Our Promise to the Community

We live in an era where people's beloved games are increasingly made unplayable
and inaccessible, when the developers decide to abandon them, thanks to online-only
gameplay and DRM. We are firmly against such practices. We believe that the fans
of a game should be able to enjoy it forever.

As such, MineWars will always provide support for LAN gameplay and community servers.

## While We Officially Support the Game

We intend to keep running official servers and to keep developing and updating
the game for as long as possible, given enough funding.

However, official servers are not the only way to play the game.

Every game install will come with a dedicated server binary, allowing anyone
to host their own unofficial servers. Though, this version of the server
may differ from the one we use for official game servers. It may have a
different feature set (missing some features we'd like to only support on
official servers, but also adding some extra features to facilitate community
server operation).

## When We No Longer Officially Support the Game

Should we go out of business or decide to abandon MineWars for any reason,
here is the plan:

We will fully open-source *everything*: client and servers. We may have to
delete some parts, if they contain secrets or other sensitive information,
but other than that, everything will be open-sourced.

## Open-Source Client Repository

Even while the game is officially supported, MineWars is a
partially-open-source project. A large part of the game client (written in
the Bevy game engine) is open-source, because we'd like the Bevy game dev
community to be able to benefit from our work: learn from how we did things
and be able to copy some of the tech we developed for MineWars.

All official game releases are proprietary. The full source code for the
official builds of the game (as distributed through official channels) is *not*
available.

However, a somewhat cut-down version of the game can be built from the public
[GitHub Repository](https://github.com/IyesGames/MineWars). All the source code
in that repository is dual-licensed as MIT/Apache2, and so is completely free
(as in freedom) for anyone to do whatever they want with it.

### What is included in the open-source repository?

It includes the majority of the game client, which is where most of the
interesting technology in MineWars is.

However, the following features are **not** included:
 - The ability to play or host the main MineWars game modes.
 - Procedurally-generated MineWars maps (for any game mode, editor, etc.)
 - Networked multiplayer
 - Anything related to integration with 3rd-party services like Steam and Discord

Notably, you **can** do the following:
 - Play Minesweeper (not MineWars) offline
 - Watch replays of MineWars games
 - Use the scenario editor to create custom maps

### Contributing

The code in the GitHub repo *is* what is actually used for official builds. It
is not a separate stripped-down code dump.

For official builds, we simply replace the `minewars-proprietary-stub` Rust dependency
with code from our private repos, to add in the additional functionality.

If you make contributions to our open-source repo, your contributions will be
included in future official updates for the game, too. :) You will be credited
for your contributions.
