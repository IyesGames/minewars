# Open-Source Builds

MineWars is a partially-open-source project.

All official game releases are proprietary. The full source code for the
official builds of the game (as distributed through official channels) is *not*
available.

However, a somewhat cut-down version of the game can be built from the public
[GitHub Repository](https://github.com/IyesGames/MineWars). All the source code
in that repository is dual-licensed as MIT/Apache2, and so is completely free
(as in freedom) for anyone to do whatever they want with it.

## What is included in the open-source version?

The open-source version actually includes most of the technology in MineWars.
The majority of the game client, and the foundations of the game servers are
open-source.

However, the following features are **not** included:
 - The ability to play or host the main MineWars multiplayer game mode
   - This includes networked play, offline play, and the tutorial
 - Procedurally-generated MineWars maps (for any game mode, editor, etc.)
 - Any form of anti-cheat
 - Access to official multiplayer servers
 - Anything related to integration with 3rd-party services like Steam and Discord

Notably, you **can** do the following:
 - Play other game modes (such as the Minesweeper-like multiplayer and singleplayer modes)
 - Connect to and play on unofficial servers that host games in other game modes
   - Even if they use the proprietary version of the server
 - Host your own game servers for other game modes, using custom maps
 - Spectate matches and watch replays of the main MineWars multiplayer game mode
 - Use the scenario editor to create custom maps (random map generation is disabled)

Basically, it provides a great starting point for developing your own
alternative game modes, if you wish to do so. Make your own modded servers with
your own custom game modes and configs, custom maps (or procedural generation
algo), etc. Learn from the code. Copypaste our code into your own gamedev
projects. Whatever. Go wild. Go nuts.

However, if you want to host unofficial servers for the main MineWars game mode,
you need the official proprietary server build, included with the official game
release. If you want to play MineWars on community servers, you need the
official game client.

The proprietary server is still decently flexible, though. It is designed to
allow the community to create unofficial infrastructure and services around it,
if you want to create your own multiplayer matchmaking platform or whatever.
You can control it via a RPC interface, to set up MineWars sessions, meaning you
could develop your own management / launcher / frontend / authentication / etc.
You can also integrate a custom anti-cheat solution.

## Contributing

The code in the GitHub repo *is* what is actually used for official builds. It
is not a separate stripped-down code dump.

For official builds, we simply replace the `minewars-proprietary-stub` Rust dependency
with code from our private repos, to add in the additional functionality.

If you make contributions to our open-source repo, your contributions will be
included in future official updates for the game, too. :) You will be credited
for your contributions.
