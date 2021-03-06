# Spectator/Replay Stream Format

The Spectator Protocol is essentially a container format that multiplexes
multiple [player protocol](./dataformat-player.md) streams (one for each player in
the game, representing their view of the world) together, along with a global
"spectator view" stream (also in the same format) providing a global view
of the game world.

This is used to give spectator clients all the data they need to simultaneously
follow all participants in the game. This is also the file format used for
replay files.

## Stream Structure

The contents of the stream/file appear in this order:

 - File Header (file only)
 - Initialization Sequence
 - [... frames ...]

## File Header

In the case of a replay file, a header is prepended.

The file header has the following structure:
 - `[u64; 3]`: checksums
 - `u16`: length of compressed frame data in bytes
 - `u16`: length of uncompressed frame data in bytes

If compressed length == uncompressed length, the frames data is stored uncompressed.

If compressed length < uncompressed length, all the frames are compressed as a single big LZ4 block.

The compression is to be done using a special dictionary, see [compression dictionary](#compression-dictionary).

## Checksums

Checksums are only used in the case of replay files. Network streams do
not have checksums. In that case, the transport protocol is assumed to be
responsible for data integrity.

The file begins with 3 SeaHash checksums.

The first checksum covers:
 - the remainder of the file header, incl. the following 2 checksums
 - the header part of the initialization sequence: everything until the city and map data

The second checksum covers:
 - the data payload of the initialization sequence: list of cities and map data

The third checksum covers:
 - all the frames data

## Initialization Sequence

This is the same as described in the [player protocol documentation](./dataformat-player.md).

## Frames

A Frame is a collection of game updates that happen on the same game tick. It
encodes the point of view of every player in the game who is involved + a
special global spectator view.  All of these "streams" are stored together
inside the frame.

Note: it is not a requirement that all game update messages from the same
tick are encoded into a single frame. They may be fragmented into multiple
frames. Subsequent frames for the same tick would just have the tick delta
field set to zero.

Such fragmentation is necessary if the updates for any included view exceed
255 bytes in length.

There are two kinds of frame encodings: Homogenous and Heterogenous.

### Homogenous Frames

Homogenous frames are frames where every participant gets the same data. The data is
only encoded once and assumed to apply to all participating streams.

Homogenous frames have the following structure:
 - `u16`: tick delta (ticks since the previous frame in the stream)
 - `u8`: length of data payload in bytes
 - `u8`: participation mask
 - [ ... data payload ... ]

The participation mask is as follows:

|Bits      |Meaning                                                  |
|----------|---------------------------------------------------------|
|`1-------`| identifies this frame as Homogenous                     |
|`-xxxxxx-`| bitmask indicating which PlayerIds the frame applies to |
|`-------x`| does the frame also apply to the global spectator view? |

The data payload is the [player protocol update messages](./dataformat-player.md#gameplay-messages).

### Heterogenous Frames

Heterogenous frames are freams where each participant gets different data. The data
for each participating stream is included in the frame.

Heterogenous frames have the following structure:
 - `u16`: tick delta (ticks since the previous frame in the stream)
 - `u8`: length of the global spectator view portion of the data payload (0 if not included in frame)
 - `u8`: participation mask
 - `[u8]`: lengths of each player view's portion of the data payload (as many as specified in the participation mask)
 - [ ... data payload ... ]

The participation mask is as follows:

|Bits      |Meaning                                                          |
|----------|-----------------------------------------------------------------|
|`0-------`| identifies this frame as Heterogenous                           |
|`-xxxxxx-`| bitmask indicating which PlayerIds the frame contains data for  |
|`-------x`| does the frame also contain data for the global spectator view? |

The data payload is the global spectator view + each player's view,
concatenated together.

Each view's data is the [player protocol update
messages](./dataformat-player.md#gameplay-messages) for that view.

The total length of the data payload is the sum of the lengths of each view's
data, as given in the Heterogenous Frame Header described above.

## The Global Spectator View

The global spectator view behaves somewhat differently from the player views.

 - No fog of war must be displayed
 - Digits are to be calculated by the client, from known mine locations

To accommodate this, there are some special provisions in the spectator
stream format, that differ from the player stream.

The initialization sequence encodes mine positions inside the map data.

The global spectator view is controlled using the same update message format
as player views, but some message types are used differently:
 - "Digit Update" is not to be used; ignore if encountered
 - "Capture + Digits" is to be interpreted as an "Ownership Change" with the PlayerId derived from the stream it occurs in.
   - Must never be encoded in Homogenous Frames with more than one PlayerId in the participation mask
 - "Road Update" is only used for acking pending roads
   - Actual conversion of tiles to road kind is done on "City Production"

## Compression Dictionary

A special dictionary is prepared to help improve compression of the update
frames. It is to be generated from the data in the initialization sequence.

It is constructed by concatenating the following data:

 - Every land coordinate on the map, in sorted order.
 - Every mountain coordinate on the map, in sorted order.
 - Some sample Homogenous Frames (tick field omitted) that are likely to occur verbatim.

The frame samples are as follows:

 - For every Player ID, in sorted order:
   - Mine Activation:
     - `00000011` `1mmmmmm1` `00000111`
   - Recover:
     - `00000001` `11111111` `10001ppp`
   - Stun:
     - `00000011` `11111111` `10000ppp`
   - Capture single tile with digit `1`:
     - `00000100` `1mmmmmm1` `11110000` `00010000`
   - Capture single tile without digit:
     - `00000011` `1mmmmmm1` `1ppp0000`

All permutations of a given sample pattern are to be concatenated, before
moving onto the next pattern.

This effectively pre-seeds the compression algorithm with data sequences
likely to occur early-game.
