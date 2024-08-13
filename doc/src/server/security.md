# Security

The Host server offers extensive security features:
 - Encryption (mandatory)
 - Client Authentication
 - IP address control
 - Player Expectation
 - Protocol abuse detection
   (rate limiting, anti-cheat, etc.)

## Encryption

All connections to the server must be encrypted using TLS (the same kind of
encryption used for HTTPS in browsers).

If you would like to use your own certificates, you can use the `mw_certgen`
CLI tool to generate and sign them.

TODO: show examples and elaborate on different possibilities and why you
might want to use non-default certificates.

## Client Authentication

Further, it is possible to configure the server to require clients to also
have a certificate. This can be used to limit clients allowed to connect.

## IP Address Control

The Host server can use an IP List file to limit which IP addresses are allowed
to connect. An IP List is just a plaintext file with one IP address per line.

It can be set to DenyList or AllowList mode. DenyList will reject the IP
addresses in the file and allow everyone else. AllowList will only allow
the IP addresses listed in the file.

IP Address Control only applies to unexpected players. Player Expectation
bypasses it. Which makes sense, because with Player Expectations, you are
effectively pre-screening your players and explicitly authorizing specific
connections.

## Player Expectation

This is the most powerful security feature. At runtime, using RPC or an Auth
server, the Host server can be told to expect specific incoming connections. It
will then accept connections if they match the expectation.

The expectation is registered for a specific IP address and can include
additional restrictions:
 - A specific TLS client certificate
 - A specific "token"/password
 - A specific Session ID to join
 - A specific Player ID to play as

When the server gets an incoming connection from the respective IP address,
it will enforce all the extra criteria. The player will only be allowed to
join if they meet all the criteria in the expectation.

Further, it is possible to disable unexpected players altogether, only
allowing clients to connect with expectations. This makes your server
inaccessible to anyone you have not explicitly authorized.

### Player Hand-Off from an Auth Server

Player Expectation is extremely powerful with an Auth server, because the
Auth server can automate it. This is called "player hand-off".

Players connect to the Auth server for services such as user accounts, lobbies,
matchmaking, etc. The Auth server decides what sessions to set up for them on
the Host server, and which Host server they will play on (if there are multiple).

When players are ready to play, the Auth server can automatically set up a
new dynamic session for them on the Host server and submit expectations for
the exact players who will play in the new session.

It can also generate single-use TLS certificates and tokens for each player,
just for that session, for additional security.

The players are then redirected to the Host server. They must connect from
the same IP address, authenticate with their newly-issued single-use
certificate and token, and join their designated session.

## Protocol Abuse Detection

After a player has already connected and joined a session, you might want to
detect and punish any abusive behavior. The Host server has an abuse-detection
subsystem, which all player input passes through. It can apply a series of
checks and issue verdicts.

The Host server we publish with the game only offers rate limiting and
rudimentary gameplay-specific checks (checking for gameplay patterns that
are obvious and blatant cheating).

Our official Host servers may have additional anti-cheat measures. We will
not disclose any details of how they work.

In the future, we may add support for custom plugins, allowing you to
integrate 3rd-party anti-cheat solutions. This is not supported yet.

The abuse detection subsystem can keep track of a "suspect score" for each
player, which is accumulated based on all the checks that are performed on the
inputs from that player. If a player is deemed suspicious enough, a punishment
can be issued, such as an automated warning, report, kick, timeout, or ban.

All of the above is configurable.

## Appendix: Understanding Server Security

To understand how to protect ourselves against attacks, we need to learn to
think like an attacker. An attacker is not concerned with what the software
is *designed* to do, they are concerned with what it *can* do. If they can
find clever and creative ways to get it to do things in a way that is not
normally intended, they could potentially use that to abuse and exploit.

### Blast Radius

The term "blast radius" refers to how many players are affected if something
goes bad (for example, if a server crashes or is successfully attacked).

MineWars Host servers, being designed to host many sessions and many players,
can have a very large blast radius. This makes them vulnerable and lucrative
to attackers.

To reduce the blast radius, consider hosting more servers with fewer sessions
each, instead of lots of sessions on fewer server. Of course, this is only
feasible if you can afford it.

### DDoS Resilience

DDoS attacks are the most commonly-attempted attacks against game servers
in general. Game servers (for any game) typically do not contain sensitive
information. If someone has an incentive to attack a server, usually it is
to try to take it down, or to otherwise worsen the experience for legitimate
players. Hence, DDoS resilience is one of the most important considerations
for game server admins and developers.

Fortunately, nowadays it is quite difficult to DDoS a server by just spamming
it with pings or other unwanted traffic. Modern computers and networks are
too fast for that; they can just drop the packets. An attacker would need
access to a very large amount of resources to successfully overwhelm a network.

Effective DDoS attacks rely on a principle called "amplification". That is:
finding a way to trick the server to spend a lot of resources with less
input. Any feature of the server which could be abused in such a way is a
DDoS risk.

The first place to look is how the server handles connection attempts.
Connecting to a MineWars server means negotiating encryption. This is an
amplification vector. An attacker could overwhelm the server with many
connection attempts that it does not intend to allow to actually connect,
causing the server to spend CPU and RAM on validating certificates, only to
have the connection closed immediately.

A MineWars Host server can be made resilient to such an attack by using
Player Expectation and disallowing unexpected players. By having players go
through an Auth server first, the Host server is protected. The Host server
simply ignores all connection attempts it does not expect.

Another DDoS vector is automatic sessions. An attacker could connect many game
clients, causing the server to create lots of sessions and run out of RAM. Do
not enable this feature on Internet-facing servers! It is intended for LANs.
Internet-facing Host servers should use either preconfigured or dynamic sessions.

After a player has actually joined a session, there is not much they can abuse,
as they cannot connect additional clients. Further, the player protocol is
monitored and rate-limited. You can adjust the rate-limiting configuration.

#### Auth Servers

The Auth server is less lucrative for DDoS attacks, because it does not affect
gameplay and is not performance sensitive. If attacked, it might only affect
players who have not yet joined a game, and the server operators. Players who
are already in-game are unaffected. To protect an Auth server, consider putting
it behind a commercial DDoS protection service, such as Cloudflare. Auth
servers are not performance sensitive and it is okay if they are accessed
through a slow proxy. Further, consider having multiple Auth servers for
redundancy.
