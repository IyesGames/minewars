# Introduction

## The Host Server

The Host server is where gameplay actually happens. This is the most important
piece of the puzzle.

It is possible to have a setup with only a Host server. Players can connect
directly to it and play the game. It might not be the most user-friendly
experience, but it will work. Notably, the Host server alone is not capable
of providing any services *outside* of the actual gameplay, such as lobbies,
accounts, matchmaking, etc. You connect and get directly thrown into a game,
that's it.

However, this is perfectly fine for many use cases. Such a configuration is
simple and easy to set up. Perfect for LAN parties, tournaments, etc.

The Host server is multi-session. One Host server instance is capable of
running many parallel gameplay sessions (possibly even thousands, depending
on your hardware).

See [Session Configuration](./sessions.md) to learn more about hosting many
sessions on one server.

## More Complex Setups

If you are really serious about MineWars server hosting and you would like
a more complex setup with extra features such as:
 - For users: services outside of gameplay: accounts, lobbies, etc.
 - For admins: management of the Host server(s): runtime configuration, session management, load balancing, telemetry, etc.

See [Runtime Management](./management.md) to learn more.

## The Config File

The Host server needs a config file. Don't worry. We provide some example
configs for you, which you can probably use as-is without modification.

See
[Tutorial and Examples](./examples.md) to get started.

## Security

MineWars *requires* encryption. Encryption requires keys and certificates.

However, don't worry about it. To make things easy for you, we provide some
default pre-made certificates that will just work out of the box. The example
config files that come with the game use them. Everything is already set up
for you.

If you are more serious about security and would like to customize things,
see the page on [Security](./security.md) to learn more. There are also
various other optional security features beyond encryption.
