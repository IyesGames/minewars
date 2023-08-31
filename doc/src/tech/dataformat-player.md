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
|`10xxxxxx`| (`x` + 12) centiseconds |
|`11xxxxxx`| (`x` + 7) deciseconds   |

**PlayerId**: a value between 1-6 inclusive.

You will also need to bring a LZ4 implementation supporting **raw blocks**
and dictionary data. The `lz4_flex` Rust crate is perfect. :)

## Initialization Sequence

When a connected player is successfully authenticated and ready to begin
the game, it will receive in **initialization sequence**, which includes
metadata about the game session, and the map data.

### Header

It begins with a header:
 - `u8`: protocol version (must be `0x01`)
 - `u8`: flags
 - `u8`: map size (radius)
 - `u8`: number of players and cities/regions
 - `u16`: length of player names data (0 for an anonymized stream)
 - `u16`: length of compressed map data in bytes
 - `u16`: length of uncompressed map data in bytes

The `flags` field is encoded as follows:

|Bits      |Meaning                     |
|----------|----------------------------|
|`----0---`| Game uses a hexagonal grid |
|`----1---`| Game uses a square grid    |
|`xxxx-xxx`|(reserved bits)             |

The player/city counts are encoded as follows:

|Bits      |Meaning                     |
|----------|----------------------------|
|`----xxxx`| Number of cities/regions   |
|`-xxx----`| Number of Players          |
|`x-------`|(reserved bits)             |

#### Game Parameters

Then follow the parameters used for the game rules, in this game.

// TODO

### Data Payload

#### Player Names

If the file is not anonymized, then follow the display names of each player,
encoded as: `u8` length in bytes, followed by UTF-8 encoded data.

#### City Locations

Then follows the list of city coordinates.

#### Map Data

Then follows the map data.

If compressed length < uncompressed length, the data is LZ4 compressed.

If compressed length == uncompressed length, the data is raw/uncompressed.

The map is encoded as one byte per tile:

|Bits      |Meaning                     |
|----------|----------------------------|
|`----xxxx`| Tile Kind                  |
|`xxxx----`| Region ID                  |

Tile Kind:
 - `0000`: Water
 - `0010`: Mountain
 - `0011`: Forest
 - `0100`: Destroyed Land
 - `0101`: Destroyed Land + Decoy
 - `0110`: Destroyed Land + Mine
 - `0111`: Destroyed Land + Flashbang
 - `1000`: Regular Land
 - `1001`: Regular Land + Decoy
 - `1010`: Regular Land + Mine
 - `1011`: Regular Land + Flashbang
 - `1100`: Fertile Land
 - `1101`: Fertile Land + Decoy
 - `1110`: Fertile Land + Mine
 - `1111`: Fertile Land + Flashbang

The "Land + Item" variants are available, so that spectator streams and replay
files do not need to start with a long sequence of messages on tick 0 to reveal
all initial item locations on the map. In regular gameplay, these will be unused.

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

|Bits      |Message Kind         |
|----------|-------------------  |
|`00000000`| Tremor              |
|`00000001`| Smoke               |
|`0000001-`| --                  |
|`000001--`| --                  |
|`00001---`| --                  |
|`00010000`| Construction Update |
|`00010---`| Player Update       |
|`00011---`| Digit Update        |
|`00100000`| Structure Gone      |
|`0010----`| Structure HP        |
|`00110---`| Explosions          |
|`00111---`| Decoys Broken       |
|`0100----`| Construction Queued |
|`0101----`| Reveal Structure    |
|`0110----`| City Status         |
|`01110---`| Tile Kind Update    |
|`011110--`| --                  |
|`011111--`| Reveal Item         |
|`1000----`| Capture + Digits    |
|`1-------`| Ownership Updates   |

The patterns must be checked in the correct order, so that more specific
bit sequences are matched first.

### Messages Documentation

Here is the complete list of game update messages and their encodings:

#### Tremor

Some explosion occurred at an unknown location. Client should shake the screen lightly.

Assembly:
```
SHAKE
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000000`| (opcode)       |

#### Smoke

A tile was smoked.

Assembly:
```
SMOKE y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00000001`| (opcode)       |

Followed by the coordinate of the tile.

#### Construction Update

Update on the progress of a pending structure.

Assembly:
```
BUILD y,x current rate
```

|Bits      |Meaning         |
|----------|----------------|
|`00010000`| (opcode)       |

Followed by the coordinates of the tile.

Followed by `u16` indicating current accumulated units.

Followed by `u16` indicating rate of construction.

#### Player Update

Something notable happened with a specific player.

Assembly:
```
PLAYER p status
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00010---`| (opcode)       |
|`-----xxx`| PlayerId       |

PlayerId must not be `000`.

The next byte specifies what happened:

|Bits      |Meaning         |
|----------|----------------|
|`00000000`| Disconnected   |
|`00000001`| Eliminated     |
|`00000010`| Stunned/Killed |
|`00000011`| Blinded        |
|`00000110`| Un-Stunned     |
|`00000111`| Un-Blinded     |
|`00001000`| Kicked         |
|...       | (reserved)     |

#### Digit Update

Update the value of the Minesweeper digit at a specific (single) tile.

Assembly:
```
DIGIT d/y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00011---`| (opcode)       |
|`-----xxx`| Digit Value    |

Followed by the coordinate of the tile.

#### Structure Gone

Something notable happened with a structure.

Assembly:
```
DECONSTRUCT y,x
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

Explosions have occurred. Tile converts to destroyed land. Any item gone.

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

The Item Kind is:
 - `0000`: Road
 - `0001`: Bridge
 - `0010`: Wall
 - `0011`: Tower
 - other values reserved

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

The Item Kind is:
 - `0000`: Road
 - `0001`: Bridge
 - `0010`: Wall
 - `0011`: Tower
 - other values reserved

Followed by tile coordinate.

#### City Update

Update on the stats of a City.

Assembly:
```
CIT i res money income
```

|Bits      |Meaning         |
|----------|----------------|
|`0110----`| (opcode)       |
|`----xxxx`| City ID        |

Followed by `u16` indicating total resources.

Followed by `u32` indicating current money.

Followed by `u16` indicating current income rate.

#### Tile Kind Update

Changes the base tile type.

Assembly:
```
TILE y,x {water|regular|fertile|destroyed|mountain|forest}
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`01110---`| (opcode)       |
|`-----xxx`| Tile Kind      |

The Tile Kind is:
 - `000`: Water
 - `001`: (reserved)
 - `010`: Mountain
 - `011`: Forest
 - `100`: Destroyed Land
 - `101`: (reserved)
 - `110`: Regular Land
 - `111`: Fertile Land

Followed by tile coordinate.

#### Reveal Item

There is an item on the specified tile.

Used both when revealing foreign items and also when acking own deployed items.

Assembly:
```
ITEM y,x {none|decoy|mine|flash}
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`111111--`| (opcode)       |
|`------xx`| Item Kind      |

The Item Kind is:
 - `00`: None
 - `01`: Decoy
 - `10`: Mine
 - `11`: Flashbang

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
|`-xxx----`| PlayerId       |
|`----xxxx`| Tile Count - 1 |

The PlayerId must not be `000`.

Followed by the coordinates of the tiles.

If any of the tiles are of a clustered tile kind (mountain, forest), it is assumed
that the ownership update applies to the entire cluster. There is no need to list
every tile coordinate of the cluster.

#### Capture + Digits

This is a more-efficient combined encoding to be used for when the player is
capturing land tiles. It can be used instead of separate "Ownership Change"
and "Digit Change" messages.

The specified tiles are now owned by the player (to whom this is addressed),
and the specified digits are to be shown on them.

Assembly:
```
DIGITS d/y,x ...
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`1000----`| (opcode)       |
|`----xxxx`| Tile Count - 1 |

Followed by the coordinates of the tiles.

Followed by the digit for each tile, two digits packed into one byte (note
big endian):

|Bits      |Meaning         |
|----------|----------------|
|`-xxx----`| digit N        |
|`-----xxx`| digit N+1      |

(this encoding allows them to be easily read when inspecting a hex dump)

For an odd number of tiles, the final digit is to be encoded as zero.

