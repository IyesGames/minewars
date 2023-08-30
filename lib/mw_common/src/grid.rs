//! IyesGames 2D Grid Library
//!
//! Library for working with small 2D grid-based game worlds.
//! Currently supports square and hexagonal maps.
//! Contains coordinate math and logic. See trait `Coord`; modules `sq`/`hex`.
//! Contains map (per-tile) data storage in different flavors. See module `map`.
//! For size compact-ness, coordinates implemented using `i8`/`u8`.
//! Designed primarily to support "radial" maps.
//!
//! Neighbor classes:
//!  - `n0`: all tiles sharing an edge
//!    - (6 for hex, 4 for square (horizontal and vertical))
//!  - `n1`: all tiles touching
//!    - (same 6 for hex, 8 for square (incl. diagonals))
//!  - `n2`: further/extended diagonals
//!    - (the 6 "hex diagonals"; "knight's move" in square)

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use enum_iterator::Sequence;
use enum_map::Enum;
use thiserror::Error;

pub mod hex;
pub mod map;
pub mod pos;
pub mod sq;

pub use hex::Hex;
pub use map::MapData;
pub use pos::Pos;
pub use sq::Sq;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Enum, Sequence)]
#[repr(u8)]
pub enum Topology {
    Hex,
    Sq,
}

/// Common interface for working with square or hex coordinates
pub trait Coord:
    Debug
    + Copy
    + Eq
    + Ord
    + Hash
    + Default
    + Send
    + Sync
    + 'static
    + Into<(i8, i8)>
    + Into<(u8, u8)>
    + From<pos::Pos>
    + Into<pos::Pos>
    + Into<glam::IVec2>
    + Into<glam::UVec2>
    + TryFrom<(f32, f32)>
    + Add<Self, Output = Self>
    + AddAssign<Self>
    + Sub<Self, Output = Self>
    + SubAssign<Self>
    + Neg
    + Mul<i8, Output = Self>
    + MulAssign<i8>
    + Mul<u8, Output = Self>
    + MulAssign<u8>
{
    const N0: usize;
    const N1: usize;
    const N2: usize;
    const TOPOLOGY: Topology;

    type IterN0: IntoIterator<Item = Self>;
    type IterN1: IntoIterator<Item = Self>;
    type IterN2: IntoIterator<Item = Self>;
    type IterRing: IntoIterator<Item = Self>;
    type IterCoords: IntoIterator<Item = Self>;

    fn origin() -> Self;
    fn distance(self, other: Self) -> u16;
    fn ring(self) -> u8;
    fn ring_len(r: u8) -> usize;
    fn iter_n0(self) -> Self::IterN0;
    fn iter_n1(self) -> Self::IterN1;
    fn iter_n2(self) -> Self::IterN2;
    fn iter_ring(self, radius: u8) -> Self::IterRing;
    fn from_f32_clamped(xy: (f32, f32)) -> Self;
    fn translation(self) -> glam::Vec2;
    fn as_pos(self) -> Pos {
        self.into()
    }
    fn map_area(r: u8) -> usize;
    fn row_len(r: u8, y: i8) -> usize;
    fn aa_indent(y: i8) -> usize;
    fn xmin(r: u8, y: i8) -> i8;
    fn xmax(r: u8, y: i8) -> i8;
    fn index(r: u8, c: Self) -> usize;
    fn iter_coords(r: u8) -> Self::IterCoords;
}

#[derive(Error, Debug)]
#[error("Out of bounds")]
pub struct OutOfBoundsError;
