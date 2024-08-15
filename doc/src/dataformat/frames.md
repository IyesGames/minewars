# Game Updates and Framing

A Frame is a collection of game updates that happen together at the same time.
It encodes the point of view of every player in the game who is involved + a
special global spectator view. All of these "streams" are stored together inside
the frame.

Note: it is not a requirement that *all* game update messages from the same
timestamp are encoded together. They may be fragmented into multiple frames.
Subsequent frames would just have their time offset set to zero.

Such fragmentation is necessary if the frame payload exceeds 256 bytes in length.

There are three kinds of frame encodings: Homogenous, Heterogenous, Keepalive.

## Homogenous Frames

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

The size of the participation mask depends on the `max_plid` field in the
Initialization Sequence. `u8` if `max_plid <= 7`, `u16` if `max_plid >= 8`.

The data payload is the [player protocol update messages](./dataformat-player.md#gameplay-messages).
All of the players listed in the participation mask must receive the entire identical data payload.

## Heterogenous Frames

Heterogenous frames are freams where each participant gets different data. The data
for each participating stream is included in the frame.

Heterogenous frames have the following structure:
 - `u16`: Header
 - `u8`/`u16`: participation mask
 - `[u8]`: lengths of each player view's portion of the data payload - 1
 - [ ... data payload ... ]

The top bit (bit 15) in the Header must be `0`, indicating that this is a Heterogenous
Frame. The remaining 15 bits represent the time delta since the previous frame, in
milliseconds, and must not be all-ones (the max value is reserved for Keepalive Frames).

The participation mask is a bitmask indicating which PlayerIds the frame applies to.
Bit 0 represents the global spectator view.

The size of the participation mask depends on the `max_plid` field in the
Initialization Sequence. `u8` if `max_plid <= 7`, `u16` if `max_plid >= 8`.

The size of the lengths array is equal to the number of `1` bits in the
participation mask. Each value represents the length of the data for that
player's view, - 1.

The data payload is the global spectator view + each player's view (in the order
of the bits in the participation mask), concatenated together.

Each view's data is the [player protocol update
messages](./dataformat-player.md#gameplay-messages) for that view.

The total length of the data payload is the sum of the lengths of each view's
data, as given in the Heterogenous Frame Header described above.

## Keepalive Frames

Keepalive frames are to be used if the time delta since the last frame is too
long to be represented in a single frame header. It is an empty frame with no
data payload, just used to advance time.

It is encoded as a frame with the time delta field being all-ones (the maximum
value). The topmost bit is unimportant/ignored.

Keepalive frames have the following structure:
 - `u16`: `-111111111111111`

Note: there is no participation mask, no data length field, no data payload
