# MineWars Data Format

The MineWars Data Format is the file format and encoding used to store MineWars
game data. Specifically, this is the format of `*.minewars` files that can
store maps and replays. It is implemented in the `mw_dataformat` Rust crate.

The Data Format is capable of storing:
 - Map Data
 - Other parameters and metadata of the game session
 - A stream of gameplay updates for all players in the game, multiplexed together,
   with timing information, to allow for watching a replay of a game.

It is not to be confused with the MineWars Player Protocol, which is what is used
over-the-wire for communication between the Game Client App and Host Server for
networked multiplayer gameplay.

The Player Protocol does internally use the Data Format for some purposes, such as:
 - Transmitting the map data and configuration metadata to start a game session (Initialization Sequence).
 - Encoding of most gameplay updates/events during gameplay (Game Update Messages).
 - Multiplexing the PoVs of all the players in the game for sending to spectators (Framing).

However, the Player Protocol also does a lot more. The full Player Protocol
is proprietary and not publicly documented.

The Player Protocol and the Data Format are versioned separately (and separately from
the MineWars client and server software), but both of their versions are important for
compatibility.

Reusing the encoding of map data and gameplay updates between all of these use cases
(live gameplay, spectation, replay files) makes it easier to implement all of this
functionality in MineWars. That is the design goal of the Data Format.

## General Properties of the Data Format

This is a custom purpose-built compact binary format.

All multi-byte values are encoded as **big endian** and unaligned.

All **coordinates** are encoded as `(row: u8, col: u8)` (note (Y,X) order).
In places where a sequence of multiple coordinates is listed, it is recommended
to encode them in sorted order. This helps compression.

Some places use a special encoding for **time durations**:

|Bits      |Meaning                  |
|----------|-------------------------|
|`0xxxxxxx`| `x` milliseconds        |
|`10xxxxxx`| (`x` + 13) centiseconds |
|`11xxxxxx`| (`x` + 8) deciseconds   |

**PlayerId**: a value between 1-15 inclusive.

**PlayerSubId**: a value between 0-14 inclusive.

You will also need to bring a LZ4 implementation supporting **raw blocks**.
The `lz4_flex` Rust crate is perfect. :)
