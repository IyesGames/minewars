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

**PlayerId**: a value between 1-6 inclusive.

You will also need to bring a LZ4 implementation supporting **raw blocks**
and dictionary data. The `lz4_flex` Rust crate is perfect. :)

## Initialization Sequence

When a connected player is successfully authenticated and ready to begin
the game, it will receive in **initialization sequence**, which includes
metadata about the game session, and the map data.

### Header

It begins with a header:
 - `u8`: flags
 - `u8`: protocol version (must be `0x01`)
 - `u8`: tick rate in Hz
 - `u8`: map size (radius)
 - `u8`: number of players
 - `u8`: number of cities/regions
 - `u16`: length of player names data (0 for an anonymized stream)
 - `u16`: length of compressed map data in bytes
 - `u16`: length of uncompressed map data in bytes

The `flags` field is encoded as follows:
|Bits      |Meaning                     |
|----------|----------------------------|
|`----0---`| Game uses a hexagonal grid |
|`----1---`| Game uses a square grid    |
|`xxxx-xxx`|(reserved bits)             |

#### Game Parameters

Then follow the parameters used for the game rules, in this game.

 - `u8`: per-city Base Resources
 - `u8`: Land resources
 - `u8`: Fertile Land resources
 - `u8`: Mountain resources
 - `u16`: (Radii)
 - `u16`: production cost of a Road
 - `u16`: production cost of a Mine
 - `u16`: production cost of a Decoy

The resources of each tile kind are packed as two 4-bit fields `eeeellll`
for the export and local resources respectively.

`Radii` is encoded as follows:

|Bits      |Meaning                                         |
|----------|------------------------------------------------|
|`-----xxx`| How many tiles player territory is fog of war? |
|`---xx---`| How many tiles from water is fertile land?     |

### Data Payload

#### Player Names

If the file is not anonymized, then follow the display names of each player,
encoded as: `u8` length in bytes, followed by UTF-8 encoded data.

#### City Locations

Then follows the list of city coordinates. Region mappings/associations are
not stored; they must be calculated/derived.

#### Map Data

Then follows the map data.

If compressed length < uncompressed length, the data is LZ4 compressed.

If compressed length == uncompressed length, the data is raw/uncompressed.

The map is encoded with a compact bit-stream encoding:

 - `00`: water tile
 - `01`: land with an initial mine on it (only used for spectator/replays)
 - `10`: mountain tile
 - `11`: land tile

The tiles are encoded in concentric-ring order, starting from the center of
the map. The map data ends when all rings up until the map radius specified in
the header have been encoded. Any final incomplete byte is padded with `0`s.

Each ring starts from the lowest (Y,X) coordinate and follows the +X direction first:

Square example (imagine the same for the hexagon equivalent):
```
v<<<
v  ^
v  ^
0>>^
```

(`0` is the starting position, assuming +X points right and +Y points up)

## Gameplay Messages

Updates for the player are encoded as a raw uncompressed block of data
consisting of any number of **messages** concatenated together. Each message
is a variable-length byte sequence.

Each message is at least one byte long. The type of the message is determined
by magic bits in that first byte (similar to opcodes in CPU instruction set
encodings). The first byte may also have bit-fields embedding data into it,
for some message types.

### Opcode Summary

Quick table summarizing the opcodes of all the message types. A few are left
unused, reserved for future use.

|Bits      |Message Kind       |
|----------|-------------------|
|`00000111`| Mine Activation   |
|`00000---`| Digit Update      |
|`00001000`| --                |
|`00001111`| --                |
|`00001---`| Player Eliminated |
|`0001----`| Road Update       |
|`001-----`| Explosion         |
|`010-----`| Defield           |
|`0110----`| --                |
|`0111----`| City Production   |
|`1000-000`| Place             |
|`1000-111`| --                |
|`10000---`| Stun              |
|`10001---`| Recover           |
|`1111----`| Capture + Digits  |
|`1-------`| Ownership Update  |

### Messages Documentation

Here is the complete list of game update messages and their encodings:

#### Mine Activation

A tile is now known to have an activated mine on it.

Assembly:
```
ACT y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000111`| (opcode)       |

Followed by the coordinate of the tile.

#### Digit Update

Update the value of the Minesweeper digit at a specific (single) tile.

Assembly:
```
DIG d/y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000---`| (opcode)       |
|`-----xxx`| Digit Value    |

Followed by the coordinate of the tile.

Digit value bitfield must never be `111`.

#### Player Eliminated

It's game over for the specified player.

Assembly:
```
RIP p
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00001---`| (opcode)       |
|`-----xxx`| PlayerId       |

PlayerId must not be `000` or `111`.

#### Road Update

If issued on own territory: ack that player has confirmed roads to be built
on the specified tiles.

If issued on foreign territory: reveal that these tiles actually have roads
built on them (convert tile kind to road).

Assembly:
```
ROAD y,x ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0001----`| (opcode)       |
|`----xxxx`| Tile Count - 1 |

Followed by the coordinates of the tiles.

#### Explosion

Mines or decoys have been destroyed at the specified tiles.

Assembly:
```
EXP [MD] y,x ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`001-----`| (opcode)       |
|`---x----`| Kind           |
|`----xxxx`| Tile Count - 1 |

The kind is `0` for mine and `1` for decoy.

Followed by the coordinates of the tiles.

#### Defield

Used to signal that mines/decoys have been removed from tiles.

If the player owns the tiles, they are assumed to be added to the player's inventory.

Used in the follwing situations:

 - The player has manually un-deployed a mine (on own tiles, no ownership change)
 - The player has captured foreign mines
   - (sent after an Ownership Update giving us ownership over the tiles)
 - The player has had their mines captured by another player
   - (sent after an Ownership Update giving another player ownership over tiles that used to be ours)

Note that this message is sensitive to ordering; its semantics may depend
on a previous Ownership Update message sent during the same game update.

Assembly:
```
REM [MD] y,x ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`010-----`| (opcode)       |
|`---x----`| Kind           |
|`----xxxx`| Tile Count - 1 |

The kind is `0` for mine and `1` for decoy.

Followed by the coordinates of the tiles.

#### City Production State

Used when a city has finished the production of an item and begun the production of another one.

Also if the active production item has been changed without completing the previous one.

Assembly:
```
PROD old new
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`0111----`| (opcode)       |
|`----xxxx`| City ID        |

Followed by another byte:

|Bits      |Meaning         |
|----------|----------------|
|`xxxx----`| Finished Item  |
|`----xxxx`| Started Item   |

The Item IDs are:

|Bits  |Meaning         |Assembly   |
|------|----------------|-----------|
|`0000`| Mine           |`MINE`     |
|`0001`| Decoy          |`DECOY`    |
|`0010`| Road           |`ROAD/y,x` |
|`1111`| *(cancel)*     |`X`        |

All other values are reserved.

The special cancel value is used in place of Finished Item if the active
production is changed, but the old item was unfinished.

Mines and decoys are implicitly assumed to be added to inventory.

If the Finished Item is Road, append coordinates.

If the Started Item is Road, append coordinates.

#### Place

Used when a player has placed a mine/decoy on a tile. Assumed to be removed from inventory.

Assembly:
```
PLC [MD] y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`1000-000`| (opcode)       |
|`----x---`| Kind           |

The kind is `0` for mine and `1` for decoy.

Followed by tile coordinate.

#### Ownership Update

Multiple tiles are now known to be owned by the specified player id.

Assembly:
```
OWN p y,x ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`1-------`| (opcode)       |
|`-xxx----`| PlayerId       |
|`----xxxx`| Tile Count - 1 |

Followed by the coordinates of the tiles.

The PlayerId must not be `000` or `111`.

#### Capture + Digits

This is a more-efficient combined encoding to be used for when the player is
capturing land tiles. It can be used instead of separate "Ownership Change"
and "Digit Change" messages.

The specified tiles are now owned by the player (to whom this is addressed),
and the specified digits are to be shown on them.

Assembly:
```
OWND d/y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`1111----`| (opcode)       |
|`----xxxx`| Tile Count - 1 |

Followed by the digit for each tile, two digits packed into one byte (note
big endian):

|Bits      |Meaning         |
|----------|----------------|
|`-xxx----`| digit N        |
|`-----xxx`| digit N+1      |

(this encoding allows them to be easily read when inspecting a hex dump)

For an odd number of tiles, the final digit is to be encoded as zero.

Followed by the coordinates of the tiles.

#### Stun (Timeout)

Specified player has been stunned and is waiting for the specified timeout.

Assembly:
```
SLEEP p time
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`10000---`| (opcode)       |
|`-----xxx`| PlayerId       |

Followed by `u16` specifying the cooldown duration in game ticks.

The PlayerId must not be `000` or `111`.

#### Recover (end of stun)

Specified player's stun has ended.

Assembly:
```
WAKE p
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`10001---`| (opcode)       |
|`-----xxx`| PlayerId       |

The PlayerId must not be `000` or `111`.
