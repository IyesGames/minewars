# Runtime Management

As discussed in the [introduction](./intro.md), the Host server alone is more than
enough for many use cases.

If you simply want to host a LAN game, or a LAN party or tournament with
many game sessions, or even a basic online server with preconfigured sessions
that people can just connect to and play, you don't need anything more than
just the Host server with an appropriate config file.

This page will teach you how to grow beyond that. How to manage your server
while it is running, to dynamically reconfigure it, set up additional
sessions, etc.

## RPC vs HostAuth

There are two ways to manage a Host server: RPC and an Auth server (HostAuth).

So, what is the difference?

RPC allows external software to connect to the server to tell it what to do.

HostAuth allows the server to connect to external software to be managed.

So the basic difference is in who initiates the connection. This makes them
suitable for different use cases.

RPC is well-suited for performing one-off actions manually and for scripting.
Connect to the server's RPC interface, send it some commands, done.

HostAuth is well-suited for automated management. It also has additional
capabilities, notably in terms of sending extra data, like:
 - Telemetry, statistics, performance monitoring
 - Reports for abusive players
 - Replays of game sessions for saving

## RPC

To use RPC, you must enable it in the config file. There are various
options available to secure it.

Please do not expose RPC to the Internet. This is a major security
risk! Restrict it to your LAN or localhost.

The easiest way to use RPC is via the `mw_hostrpc` CLI tool. It lets you
just send commands. It is also designed to be suitable for scripting. It
can also just generate the message and output it to you instead of sending it.

The RPC protocol itself is based on the RON format and is human-readable
and writable. If `mw_hostrpc` doesn't do something you need, or you need
low-level control, you *could* just write your own messages and send them
via netcat or something. Or more likely, generate something with `mw_hostrpc`
and then edit it.

## The Auth Server

The Auth Server is an optional extra server you can run to manage your Host
server(s), specifically the sessions on them and the players connecting
to them.

It is responsible for dealing with your users *outside* of gameplay.

It can offer various (optional) features to players, including:
 - Listing of available sessions they can join
 - Letting them choose what session to join
 - Letting them see info about the players in different sessions
 - Letting them see info about other players who have not yet joined a session.
 - Letting them enter a session as a lobby with specific other players.
 - Letting them create their own sessions
 - Letting them create custom game configurations

Basically, a pre-game lobby sort of menu interface.

To you, the admin, it can also offer some compelling features:
 - [Increased security for your Host server(s)](./security.md#player-expectation)
 - Monitoring of multiple Host servers
 - Automatically selecting an appropriate Host for players, based on their ping, server load, etc.

Basically the job of the Auth server is to have players connect to it first,
instead of connecting directly to the Host server. It can let players decide
how they want to play the game and who they want to play with. Then, it can
actually set up a session for them on a Host server and redirect them to it.

It also allows you to control who you allow to play on your servers.

### MiniAuth

Your game install comes with MiniAuth, a limited version of the Auth
server we use for the MineWars official multiplayer. It offers all of
the above features. However, it does not have "account services" (user
accounts/login). It cannot support integrations and authentication with
proprietary platforms like Steam. That is the only difference from the
official version of the Auth server we use internally.

### Setting Up MiniAuth

Similar to the Host server, you basically need to have a config file and
then run `mw_miniauth`. We provide example config files you can use as a
starting point.

```sh
./mw_miniauth --config cfg/examples/auth/simple.toml
```

To actually use it, you need to also configure your Host server(s) to connect
to it. The settings are in the `[hostauth]` section of the Host server
config file.

The Host server will connect to the Auth server to be managed.

A Host server can be configured to connect to multiple Auth servers for
redundancy. All of them will have equal capabilities. Any of them can create
sessions and hand-off players to it.

Now that you have an Auth server, you probably also want to change some other
things in your Host server config:
 - Disable unexpected players, to allow players to only connect via the Auth server
 - Disable any preconfigured and automatic sessions. Put your session presets in the Auth server config instead.

That is, unless, for whatever reason, you want to still allow players to
connect to your Host server directly. You could have a "hybrid" setup,
where you have a Host server with some preconfigured sessions and also have
the Auth server set up additional sessions. If players connect to your Host
server, they are treated as "unexpected" and join the fixed sessions. If
they connect via the Auth server, they will be treated as "expected" and
join the session the Auth server set up for them.

Why you would want such a configuration, I don't know … :D

### The User Experience

Your players should connect to your Auth server instead of your Host server.
The MineWars game client is capable of automatically detecting what kind of
server it is connecting to. Users don't need to do anything different.

From a user's perspective: if they connect to a Host server, gameplay starts
immediately. If they connect to an Auth server, they are presented with a
menu and lobby interface first, if any of those features are enabled on the
Auth server. That is the difference in the user experience.

If none of the special lobby / session browser servers are enabled on the Auth,
the game client just gets automatically redirected to the Host, and gameplay
starts immediately, just as if it had connected directly to the Host.
