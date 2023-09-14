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

However, starting item positions should be encoded inside the map data.

## Frames

A Frame is a collection of game updates that happen together at the same time.
It encodes the point of view of every player in the game who is involved + a
special global spectator view. All of these "streams" are stored together inside
the frame.

Note: it is not a requirement that *all* game update messages from the same
timestamp are encoded together. They may be fragmented into multiple frames.
Subsequent frames would just have the timestamp field set to zero.

Such fragmentation is necessary if the frame payload exceeds 256 bytes in length.

There are three kinds of frame encodings: Homogenous, Heterogenous, Keepalive.

### Homogenous Frames

Homogenous frames are frames where every participant gets the same data. The data is
only encoded once and assumed to apply to all participating streams.

Homogenous frames have the following structure:
 - `u16`: Header
 - `u8`/`u16`: participation mask
 - `u8`: length of data payload in bytes - 1
 - [ ... data payload ... ]

The top bit (bit 15) in the Header must be `1`, indicating that this is a Homogenous
Frame. The remaining 15 bits represent the time delta since the previous frame, in
milliseconds, and must not be all-ones (the max value is reserved for Keepalive Frames).

The participation mask is a bitmask indicating which PlayerIds the frame applies to.
Bit 0 represents the global spectator view.

The size of the participation mask is determined by the "max player id" bit in the
Initialization Sequence.

The data payload is the [player protocol update messages](./dataformat-player.md#gameplay-messages).
All of the players listed in the participation mask must receive the entire identical data payload.

### Heterogenous Frames

Heterogenous frames are freams where each participant gets different data. The data
for each participating stream is included in the frame.

Heterogenous frames have the following structure:
 - `u16`: Header
 - `u8`/`u16`: participation mask
 - `[u8]`: lengths of each player view's portion of the data payload (as many as specified in the participation mask)
 - [ ... data payload ... ]

The top bit (bit 15) in the Header must be `0`, indicating that this is a Heterogenous
Frame. The remaining 15 bits represent the time delta since the previous frame, in
milliseconds, and must not be all-ones (the max value is reserved for Keepalive Frames).

The participation mask is a bitmask indicating which PlayerIds the frame applies to.
Bit 0 represents the global spectator view.

The size of the participation mask is determined by the "max player id" bit in the
Initialization Sequence.

The data payload is the global spectator view + each player's view (in the order
of the bits in the participation mask), concatenated together.

Each view's data is the [player protocol update
messages](./dataformat-player.md#gameplay-messages) for that view.

The total length of the data payload is the sum of the lengths of each view's
data, as given in the Heterogenous Frame Header described above.

### Keepalive Frames

Keepalive frames are to be used if the time delta since the last frame is too
long to be represented in a single frame header. It is an empty frame with no
data payload, just used to advance time.

It is encoded as a frame with the time delta field being all-ones (the maximum
value). The topmost bit is unimportant/ignored.

Keepalive frames have the following structure:
 - `u16`: `-111111111111111`

Note: there is no participation mask, no data length field, no data payload

## The Global Spectator View

The global spectator view behaves somewhat differently from the player views.

 - No fog of war must be displayed
 - Digits are to be calculated by the client, from known mine locations

To accommodate this, there are some special provisions in the spectator
stream format, that differ from the player stream.

The initialization sequence encodes mine positions inside the map data.

The global spectator view is controlled using the same update message format
as player views, but some message types are used differently:
 - "Digit Update" and "Capture + Digits" must have the tile owner inferred
   from the participation mask. The mask must encode only one PlayerID
   (other than bit 0 for the spectator stream).

## Compression Dictionary

A special dictionary is prepared to help improve compression of the update
frames. It is to be generated from the data in the initialization sequence.

It is constructed by concatenating the following data:

 - Every mountain coordinate on the map, in sorted order.
 - Every land coordinate on the map, in sorted order.

All permutations of a given sample pattern are to be concatenated, before
moving onto the next pattern.

This effectively pre-seeds the compression algorithm with data sequences
likely to occur early-game.
