# File Structure

A MineWars File contains the following, in order:
 - [File Header](#file-header)
 - [Initialization Sequence](./is.md)
 - [Frames of Game Updates](./frames.md)

## File Header

The file header has the following structure:
 - `[u64; 3]`: checksums
 - `u32`: length of compressed frame data in bytes
 - `u32`: length of uncompressed frame data in bytes

If compressed length == uncompressed length, the frames data is stored uncompressed.

If compressed length < uncompressed length, all the frames are compressed as a single big LZ4 block.

### Checksums

The file begins with 3 SeaHash checksums.

The first checksum covers:
 - the remainder of the file header, incl. the following 2 checksums
 - the header part of the [Initialization Sequence](./is.md)

The second checksum covers:
 - the data of the Initialization Sequence (everything after the header)

The third checksum covers:
 - all the frames data

## Initialization Sequence

After the File Header follows the [Initialization Sequence](./is.md).

## Frames

After the IS follow [frames of game updates](./frames.md)

Note: neither the length of the IS nor the start offset of the frame data
are encoded in the file header. The IS Header must be parsed to compute that.

It is thus impossible to read the frames from a MineWars file without
decoding the IS first.
