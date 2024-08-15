# Initialization Sequence (IS)

The IS is what sets the general configuration and metadata of the game and
encodes the initial state of map that the game will be played on.

## Header

It begins with a header:
 - (`u8`,`u8`,`u8`,`u8`): Data Format Version
 - `u8`: flags
 - `u8`: map size (radius)
 - `u8`: `max_plid` (bits 0-3), `max_sub_plid` (bits 4-7)
 - `u8`: number of cities/regions
 - `u32`: length of compressed map data in bytes
 - `u16`: length of the Rules data
 - `u16`: length of the Cits names data

The `flags` field is encoded as follows:

|Bits      |Meaning                     |
|----------|----------------------------|
|`----0---`| Game uses a hexagonal grid |
|`----1---`| Game uses a square grid    |
|`xxx--xxx`|(reserved bits)             |

## Map Data

Then follows the map data.

If compressed length < uncompressed length, the data is LZ4 compressed.

If compressed length == uncompressed length, the data is raw/uncompressed.

The compressed length is stored in the header. The uncompressed length must
be computed from the map radius.

First, the map data is encoded as one byte per tile:

|Bits      |Meaning                     |
|----------|----------------------------|
|`----xxxx`| Tile Kind                  |
|`xxxx----`| Item Kind                  |

Tile Kind: same encoding as the "Tile Kind Update" message below.
Item Kind: same encoding as the "Reveal Item" message below.

The Item Kind is useful for spectators and replay files, so that they don't
need to start with a long sequence of "Reveal Item" messages at tick 0 for
all the initial items on the map. Other use cases (such as "map files")
may just always set it to zero.

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

If the number of cities/regions is 0, this part of the map data is skipped.

## City Info

First, locations for each city on the map:
 - `(u8, u8)`: (y, x) location

Then, names for each city on the map:
 - `u8`: length in bytes
 - …: phonemes

The name uses a special Phoneme encoding (undocumented, see source code),
which can be rendered/localized based on client language.

## Game Parameters / Rules

Then follow the parameters used for the game rules, in this game.

// TODO
