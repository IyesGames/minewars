# Game Update Messages

Updates for the players are encoded as any number of **messages** concatenated
together. Each message is a variable-length byte sequence.

Each message is at least one byte long. The type of the message is determined
by magic bits in that first byte (similar to opcodes in CPU instruction set
encodings). The first byte may also have bit-fields embedding data into it,
for some message types.

### Opcode Summary

Quick table summarizing the opcodes of all the message types. A few are left
unused, reserved for future use.

|Bits      |Message Kind         |
|----------|---------------------|
|`00000000`| Player Update       |
|`00000001`| Tremor              |
|`00000010`| Smoke Start         |
|`00000011`| Smoke End           |
|`00000100`| City MoneyInfo      |
|`00000101`| City Transaction    |
|`00000110`| City ResInfo        |
|`00000111`| City TradeInfo      |
|`000010--`| --                  |
|`0000110-`| --                  |
|`00001110`| Debug               |
|`00001111`| Flag State          |
|`0001----`| Reveal Item         |
|`00100000`| Structure Gone      |
|`0010----`| Structure HP        |
|`0011----`| Explosions          |
|`0100----`| Construction Queued |
|`01001111`| Construction Update |
|`0101----`| Reveal Structure    |
|`01011111`| --                  |
|`0110----`| Digits (single)     |
|`0111----`| Tile Kind Update    |
|`1000----`| Digits (multi)      |
|`1-------`| Ownership Updates   |

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

|Bits      |Meaning         |Granularity|Assembly                    |Class        |
|----------|----------------|-----------|----------------------------|-------------|
|`00000000`| Joined         |PlayerSubId|`JOIN`                      |Notification |
|`00000001`| Ping/RTT Info  |PlayerSubId|`RTT millis`                |Unreliable   |
|`00000010`| Timeout        |Either     |`TIMEOUT millis`            |Notification |
|`00000011`| TimeoutDone    |Either     |`RESUME`                    |Notification |
|`00000100`| Exploded       |Either     |`EXPLODE y,x killer`        |Notification |
|`00000101`| LivesRemain    |Either     |`LIVES n`                   |Notification |
|`00000110`| Protected      |PlayerId   |`PROTECT`                   |Notification |
|`00000111`| Un-Protected   |PlayerId   |`UNPROTECT`                 |Notification |
|`00001000`| Eliminated     |PlayerId   |`ELIMINATE`                 |Notification |
|`00001001`| Surrendered    |PlayerId   |`SURRENDER`                 |Notification |
|`00001010`| Disconnected   |PlayerSubId|`LEAVE`                     |Notification |
|`00001011`| Kicked         |PlayerSubId|`KICK`                      |Notification |
|`00010010`| MatchTimeRemain|Either     |`TIMELIMIT secs`            |Notification |
|`10001000`| Capturing City |Either     |`CITCAPTING citid millis`   |PvP          |
|`10001001`| Capture City   |Either     |`CITCAPTURE citid`          |PvP          |
|`10001010`| Contested City |Either     |`CITCONTEST citid`          |PvP          |
| ...      | (reserved)     |           |                            |             |

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

#### Debug

Special message reserved for use during development.

Assembly:
```
DEBUG i y,x
```

Encoding:

|Bits      |Meaning         |
|----------|----------------|
|`00001110`| (opcode)       |

Followed by:

 - `u8` magic value
 - Tile coordinate

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
|`1---0000`| (opcode)       |
|`-xxx----`| Tile Count - 1 |

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
|`----xxxx`| PlayerId       |
|`-xxx----`| Tile Count - 1 |

The PlayerId must not be zero.

Followed by the coordinates of the tiles.

If any of the tiles are of a clustered tile kind (mountain, forest), it is assumed
that the ownership update applies to the entire cluster. There is no need to list
every tile coordinate of the cluster.
