# Player Stream Format

This page describes all the encodings used for data sent from the server to
the client.

This message format is also used inside of the [spectator/replay
message](./dataformat-spectator.md) format (which encapsulates multiple streams of
this player message format).

---

## Prerequisites for Implementation

This is a custom purpose-built compact binary format.

All multi-byte values are encoded as **big endian** and unaligned.

All **coordinates** are encoded as `(row: u8, col: u8)` (note (Y,X) order).
In places where a sequence of multiple coordinates is listed, it is recommended
to encode them in sorted order. This helps compression.

All **time durations** are encoded as:

|Bits      |Meaning                  |
|----------|-------------------------|
|`0xxxxxxx`| `x` milliseconds        |
|`10xxxxxx`| (`x` + 13) centiseconds |
|`11xxxxxx`| (`x` + 8) deciseconds   |

**PlayerId**: a value between 1-15 inclusive.

You will also need to bring a LZ4 implementation supporting **raw blocks**
and dictionary data. The `lz4_flex` Rust crate is perfect. :)

## Initialization Sequence

When a connected player is successfully authenticated and ready to begin
the game, it will receive in **initialization sequence**, which includes
metadata about the game session, and the map data.

### Header

It begins with a header:
 - (`u8`,`u8`,`u8`,`u8`): protocol version (must be `(0, 1, 0, 0)`)
 - `u8`: flags
 - `u8`: map size (radius)
 - `u8`: number of players
 - `u8`: number of cities/regions
 - `u32`: length of compressed map data in bytes
 - `u16`: length of the Rules data
 - `u16`: length of the Cits names data
 - `u16`: length of the player names data (0 for an anonymized stream)
 - `u16`: (reserved)

The `flags` field is encoded as follows:

|Bits      |Meaning                     |
|----------|----------------------------|
|`----0---`| Game uses a hexagonal grid |
|`----1---`| Game uses a square grid    |
|`xxx--xxx`|(reserved bits)             |

#### Map Data

Then follows the map data.

If compressed length < uncompressed length, the data is LZ4 compressed.

If compressed length == uncompressed length, the data is raw/uncompressed.

First, the map data is encoded as one byte per tile:

|Bits      |Meaning                     |
|----------|----------------------------|
|`-----xxx`| Tile Kind                  |
|`-xxx----`| Item Kind                  |

Tile Kind: same encoding as the "Tile Kind Update" message below.
Item Kind: same encoding as the "Reveal Item" message below.

The Item Kind is only used for spectator streams and replay files, so that they
don't need to start with a long sequence of "Reveal Item" messages at tick 0
for all the initial items on the map. In player streams, this field should be 0.

If any starting Structures must be encoded (say for a custom game mode / scenario),
initialize them using regular gameplay messages at tick 0.

The tiles are encoded in concentric-ring order, starting from the center of
the map. The map data ends when all rings up until the map radius specified in
the header have been encoded.

Each ring starts from the lowest (Y,X) coordinate and follows the +X direction first:

Square example:
```
654
7.3
012
```

Hex example:
```
 4 3
5 . 2
 0 1
```

(`0` is the starting position, assuming +X points right and +Y points up)

After the map data, regions are encoded the same way: one byte per tile, in
concentric ring order. The byte is the city/region ID for that tile.

### City Info

First, locations for each city on the map:
 - `(u8, u8)`: (y, x) location

Then, names for each city on the map:
 - `u8`: length in bytes
 - …: phonemes

The name uses a special Phoneme encoding (undocumented, see source code),
which can be rendered/localized based on client language.

### Player Names

If the file is not anonymized, then follow the display names of each player,
encoded as: `u8` length in bytes, followed by UTF-8 encoded data.

### Game Parameters / Rules

Then follow the parameters used for the game rules, in this game.

// TODO

## Gameplay Messages

Updates for the player are encoded as a raw uncompressed block of data
consisting of any number of **messages** concatenated together. Each message
is a variable-length byte sequence.

Each message is at least one byte long. The type of the message is determined
by magic bits in that first byte (similar to opcodes in CPU instruction set
encodings). The first byte may also have bit-fields embedding data into it,
for some message types.

### Message Classes

This affects how messages should be sent over the network transport protocol.
It is valid (though suboptimal) to play the game over a single TCP stream
(reliable, ordered delivery of all messages). However, for optimal performance,
a protocol like QUIC, that allows more granular control of ordering and
reliability, should be used.

There are five classes of messages: PvP, Notification, Personal, Background, Unreliable.
Messages can be freely "upgraded" to a higher class; that is, if there is a queue/buffer
of messages to send, which contains messages from multiple different classes, they can
all be bundled together and sent over the highest-class stream.

PvP messages are all game updates that are part of a player's interaction with
the game world and other players. Reliable, ordered, elevated (highest)
priority.

Notification messages inform the client about important events, but are not
directly part of moment-to-moment gameplay. Reliable, ordered, medium priority.

Personal messages are all game updates that are part of game mechanics internal
within a player's own territory. Things that only directly affect them.
Reliable, ordered, lower priority.

Background messages are things that are not an interactive part of gameplay.
Reliable, unordered, lowest priority.

Unreliable messages are realtime updates that are fine to miss. Can be sent as
datagrams. They can also be omitted from replay files / spectation.

### Opcode Summary

Quick table summarizing the opcodes of all the message types. A few are left
unused, reserved for future use.

|Bits      |Message Kind         |Class                           |
|----------|---------------------|---                             |
|`00000000`| Player Update       | Notification                   |
|`00000001`| Tremor              | Background                     |
|`00000010`| Smoke Start         | PvP                            |
|`00000011`| Smoke End           | PvP                            |
|`00000100`| City MoneyInfo      | Unreliable                     |
|`00000101`| City Transaction    | Personal                       |
|`00000110`| City ResInfo        | Personal                       |
|`00000111`| City TradeInfo      | Personal                       |
|`000010--`| --                  |                                |
|`0000110-`| --                  |                                |
|`00001110`| --                  |                                |
|`00001111`| Flag State          | PvP                            |
|`0001----`| Reveal Item         | PvP (foreign), Personal (own)  |
|`00100000`| Structure Gone      | PvP, Personal (cancel pending) |
|`0010----`| Structure HP        | PvP                            |
|`0011----`| Explosions          | PvP                            |
|`0100----`| Construction Queued | Personal                       |
|`01001111`| Construction Update | Unreliable                     |
|`0101----`| Reveal Structure    | PvP                            |
|`01011111`| --                  |                                |
|`0110----`| Digits (single)     | PvP                            |
|`0111----`| Tile Kind Update    | PvP                            |
|`1000----`| Digits (multi)      | PvP                            |
|`1-------`| Ownership Updates   | PvP                            |

The patterns must be checked in the correct order, so that more specific
bit sequences are matched first.

### Messages Documentation

Here is the complete list of game update messages and their encodings:

#### Player Update

Something notable happened with a specific player.

Assembly:
```
PLAYER plid status ...
```
```
PLAYER plid/sub status ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000000`| (opcode)       |

The next byte:

|Bits      |Meaning         |
|----------|----------------|
|`----xxxx`| PlayerId       |
|`xxxx----`| PlayerSubId    |

PlayerId is the gameplay plid (view) that is affected.

PlayerSubId is the individual user/client, in game modes where multiple people
can control a single in-game plid.

For messages that apply to all PlayerSubIds of a given PlayerId,
the PlayerSubId field must be all-ones.

The next byte specifies the message kind (what happened):

|Bits      |Meaning         |Granularity|Assembly             |
|----------|----------------|-----------|---------------------|
|`00000000`| Joined         |PlayerSubId|`JOIN`               |
|`00000001`| Ping/RTT Info  |PlayerSubId|`RTT millis`         |
|`00000010`| Timeout        |Either     |`TIMEOUT millis`     |
|`00000011`| TimeoutDone    |Either     |`RESUME`             |
|`00000100`| Exploded       |Either     |`EXPLODE y,x killer` |
|`00000101`| LivesRemain    |Either     |`LIVES n`            |
|`00000110`| Protected      |PlayerId   |`PROTECT`            |
|`00000111`| Un-Protected   |PlayerId   |`UNPROTECT`          |
|`00001000`| Eliminated     |PlayerId   |`ELIMINATE`          |
|`00001001`| Surrendered    |PlayerId   |`SURRENDER`          |
|`00001010`| Disconnected   |PlayerSubId|`LEAVE`              |
|`00001011`| Kicked         |PlayerSubId|`KICK`               |
|`00001100`| Vote No        |PlayerSubId|`VOTE N`             |
|`00001101`| Vote Yes       |PlayerSubId|`VOTE Y`             |
|`00001110`| Vote Failed    |PlayerSubId|`VOTEFAIL`           |
|`00001111`| Vote Success   |PlayerSubId|`VOTEPASS`           |
|`00010000`| Chat (All)     |PlayerSubId|`CHATALL`            |
|`00010001`| Chat (Friendly)|PlayerSubId|`CHAT string`        |
|`00010010`| MatchTimeRemain|Either     |`TIMELIMIT secs`     |
|`00010011`| Initiate Vote  |PlayerSubId|`VOTENEW string`     |
| ...      | (reserved)     |           |                     |

Then follows the data payload for the given message kind.

#### Tremor

Some explosion occurred at an unknown location. Client should shake the screen lightly.

Assembly:
```
SHAKE
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000001`| (opcode)       |

#### Smoke Start

A tile was smoked.

Assembly:
```
SMOKE y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000010`| (opcode)       |

Followed by the coordinate of the tile.

#### Smoke End

A tile is no longer smoked.

Assembly:
```
UNSMOKE y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000011`| (opcode)       |

Followed by the coordinate of the tile.

#### Flag State

Report the presence or absence of a flag on a given tile.

Assembly:
```
FLAG p y,x
```
(p == plid, 0 == no flag)

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00001111`| (opcode)       |

Followed by:

|Bits      |Meaning         |
|----------|----------------|
|`0000----`| (reserved)     |
|`----pppp`| PlayerId       |

Reserved bits must be zero.

`p` is the PlayerId that placed the flag. Zero means no flag.

Followed by the coordinate of the tile.

#### Construction Update

Update on the progress of a pending structure.

Assembly:
```
BUILD y,x current rate
```

|Bits      |Meaning         |
|----------|----------------|
|`01001111`| (opcode)       |

Followed by the coordinates of the tile.

Followed by `u16` indicating current accumulated units.

Followed by `u16` indicating rate of construction.

#### Digits (+ Implied Capture)

The specified tiles are owned by the player and display the given Minesweeper digit.

Can be used to capture tiles, if the tile is not owned by the player.

Can be used to update digits on owned tiles, when they change.

Assembly:
```
DIGITS d/y,x ...
DIGITS d*/y,x ...
```

Compact (single) Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0110----`| (opcode)       |
|`----x---`| Asterisk       |
|`-----xxx`| Digit Value    |

Followed by the coordinate of the tile.

Multi-tile Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`1000----`| (opcode)       |
|`----xxxx`| Tile Count - 1 |

Followed by the coordinates of the tiles.

Followed by the digit for each tile, two digits packed into one byte:

|Bits      |Meaning         |
|----------|----------------|
|`x-------`| asterisk N     |
|`-xxx----`| digit N        |
|`----x---`| asterisk N+1   |
|`-----xxx`| digit N+1      |

(this encoding allows them to be easily read when inspecting a hex dump)

For an odd number of tiles, the final digit is ignored (should be encoded as zero).

#### Structure Gone

The structure on the given tile is removed.

Used when a built structure is destroyed or bulldozed.
Used when a pending (unbuilt) structure is canceled.

Assembly:
```
NOSTRUCT y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00100000`| (opcode)       |

Followed by the tile coordinates.

#### Structure HP

The HP of a structure changed.

Assembly:
```
STRUCTHP y,x hp
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0010----`| (opcode)       |
|`----xxxx`| HP             |

HP must be > 0.

Followed by the tile coordinates.

#### Explosions

Explosions have occurred.

If the client should know what item was destroyed, send a "Reveal Item" first.

Assembly:
```
EXPLODE y,x ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0011----`| (opcode)       |
|`----xxxx`| Tile Count - 1 |

Followed by the coordinates of the tiles.

#### Construction Queued

A new structure is pending construction.

Assembly:
```
BUILDNEW y,x {road|bridge|wall|tower}
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0100----`| (opcode)       |
|`----xxxx`| Structure Kind |

The Structure Kind is:
 - `0000`: Road
 - `0001`: Bridge
 - `0010`: Wall
 - `0011`: Tower
 - other values reserved

Must not be `1111`.

Followed by tile coordinate.

Followed by `u16` indicating total points required to complete construction.

#### Reveal Structure

There is a structure on the specified tile.

Used both when revealing foreign structures and also when own structures finish construction.

Assembly:
```
STRUCT y,x {road|bridge|wall|tower}
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0101----`| (opcode)       |
|`----xxxx`| Structure Kind |

The Structure Kind is:
 - `0000`: Road
 - `0001`: Bridge
 - `0010`: Wall
 - `0011`: Tower
 - other values reserved

Must not be `1111`.

Followed by tile coordinate.

#### City MoneyInfo

Reports how much money a city has.

Assembly:
```
CITMONEY i money
```
```
CITINCOME i money income
```

|Bits      |Meaning         |
|----------|----------------|
|`00000100`| (opcode)       |

Followed by:
 - `u8`: City ID
 - `u32`: current money
 - [`u16`: current income rate]

The top bit (bit 31) of money indicates whether the
income is reported too. The income field is only
present if this bit is `1`.

The remaining 31 bits are used for the money value.

#### City Transaction

Reports that a city has gained or spent a given sum of money.

Assembly:
```
CITTRANS i spent
```

|Bits      |Meaning         |
|----------|----------------|
|`00000101`| (opcode)       |

Followed by:
 - `u8`: City ID
 - `i16`: the amount of money

#### City ResInfo

Update on the resources of a city.

Assembly:
```
CITRES i res
```

|Bits      |Meaning         |
|----------|----------------|
|`00000110`| (opcode)       |

Followed by:
 - `u8`: City ID
 - `u16`: the amount of resources

#### City TradeInfo

Update on the export/import policy of a city.

Assembly:
```
CITTRADE export import
```

|Bits      |Meaning         |
|----------|----------------|
|`00000111`| (opcode)       |

Followed by:
 - `u8`: City ID
 - `u8`: Export rate
 - `u8`: Import rate

#### Tile Kind Update

Changes the base tile type.

Assembly:
```
TILE y,x {water|regular|fertile|destroyed|foundation|mountain|forest}
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0111----`| (opcode)       |
|`----xxxx`| Tile Kind      |

The Tile Kind is:
 - `0000`: Water
 - `0001`: (reserved)
 - `0010`: Mountain
 - `0011`: Forest
 - `0100`: Destroyed Land
 - `0101`: Foundation
 - `0110`: Regular Land
 - `0111`: Fertile Land
 - ...   : (reserved)

Followed by tile coordinate.

#### Reveal Item

There is an item on the specified tile.

Used both when revealing foreign items and also when acking own deployed items.

Assembly:
```
ITEM y,x {none|decoy|mine|trap}
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0001----`| (opcode)       |
|`----xxxx`| Item Kind      |

The Item Kind is:
 - `0000`: None
 - `0001`: Decoy
 - `0010`: Mine
 - `0011`: Trap
 - ...   : (reserved)

Followed by tile coordinate.

#### Ownership Update

Multiple tiles are now known to be owned by the specified player id.

Assembly:
```
OWNER p y,x ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`1-------`| (opcode)       |
|`-xxxx---`| PlayerId       |
|`-----xxx`| Tile Count - 1 |

The PlayerId must not be zero.

Followed by the coordinates of the tiles.

If any of the tiles are of a clustered tile kind (mountain, forest), it is assumed
that the ownership update applies to the entire cluster. There is no need to list
every tile coordinate of the cluster.
