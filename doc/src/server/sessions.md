# Session Configuration

The Host server is quite flexible with its session management. Some of the possibilities include:
 - Any number of preconfigured sessions. Specify what you want in the config file.
    - Multiple "presets" for different kinds of sessions, game modes, etc.
    - Specify how many you want from each preset.
 - Additional sessions created automatically, as players join. Based on a "preset" you specify in the config file.
    - If the server is full.
    - If a player connects and requests to join a session ID that does not exist.
 - Additional sessions created dynamically at runtime, upon request. See [Runtime Management](./management.md).
 - Any combination of the above.

## Session IDs

Every session that is currently running on your Host server has a Session ID.

Upon connecting, players can specify which Session ID they would like to join.

If they don't specify anything, the server will automatically pick a session
for them. This means you could set up a community server with multiple sessions
and people will just join a random available session upon connecting by default.

Having your players enter a special magic number to enter a specific session
is not a very user-friendly experience. Though it is fine for LANs and such.

If you would like to provide your users with a better experience, you need
to also set up an [Auth Server](./management.md#the-auth-server).

## Presets

In your config file, you can create any number of "presets", where you specify
session parameters (like the game mode, number of players, map to play on, etc.).

The number of players for the session is specified using the `wait_plids`,
`wait_subplids`, `max_plids`, `max_subplids` options.

"Plids" are the "logical" players. Think each of the different
territories/empires in a MineWars game.

"SubPlids" are the clients controlling each Plid. Use this to create sessions
where multiple people control the same in-game territory. The value is per-Plid.

The `wait_*` options specify how many players to require to join the session
upfront, before gameplay begins.

The `max_*` options specify the maximum number of players that can join
the session.  If greater than the respective `wait_*` option, additional
players may join mid-game. If less than the respective `wait_*` option,
will override it.

You can also configure a session to auto-restart after game over. This is useful
for preconfigured sessions.

### Maps

You can play on either procedurally-generated maps or pre-existing maps
loaded from files.

#### Generated Maps

If you would like to play on a procedurally-generated map, you can configure
the parameters like map size, number of cities, etc.

For Minesweeper game modes, you can also play on `Flat` maps. This is a classic
simple grid with no special features (no geography, no cities). MineWars
game modes cannot be played on `Flat` maps.

#### Map Files

If you would like to play on a custom pre-existing map, specify the path
to a map file to load.

Maps loaded from files are cached. Each file path will only be loaded once
and stored in RAM. If you have many sessions using the same map file, it
will not need to be loaded again. However, the server will detect if the
map file is modified and attempt to automatically reload it if so. The
new map data will only apply to any new sessions started after reloading.

## Preconfigured Sessions

You can configure a fixed number of sessions to run using your various presets.

These sessions will be set up as soon as you launch the Host server.

If you'd like the server to keep running and start a new game after game over,
enable autorestart in the presets you use for your preconfigured sessions.

Otherwise, after all sessions finish, if [Automatic
Sessions](#automatic-sessions) and [Runtime Management](./management.md)
is disabled, the server will shut down, as there is nothing more to do.

## Automatic Sessions

You can configure the server to automatically set up new sessions when
players connect.

**WARNING:** this feature is intended for LANs and other safe environments.
Do not enable Automatic Sessions on Internet-facing servers! It is a major
DDoS risk!

If players connect without specifying a Session ID, and there are no sessions
available for them to join, the server will create a new session using the
preset you specify in the `autosession` option.

If players connect and want to join a specific Session ID that does not
exist on the server, the server will create a new session using the preset
you specify in the `autosession_on_request` option.

## Dynamic Sessions

Additional sessions can be created at runtime. See [Runtime Management](./management.md).
