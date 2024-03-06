use derive_more::*;

use std::ops::{Mul, MulAssign};

use crate::prelude::*;
use super::Coord;

/// Topology-agnostic (type erased) coordinate
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Add, AddAssign, Sub, SubAssign, Neg,
)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Serialize, Deserialize)]
#[serde(from = "(i8, i8)", into = "(i8, i8)")]
#[repr(C)]
pub struct Pos(pub i8, pub i8);

impl Pos {
    pub fn origin() -> Self {
        Pos(0, 0)
    }
    pub fn x(self) -> i8 {
        self.1
    }
    pub fn y(self) -> i8 {
        self.0
    }
}

impl Mul<i8> for Pos {
    type Output = Pos;

    fn mul(self, s: i8) -> Pos {
        let y = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let x = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Pos(y, x)
    }
}

impl MulAssign<i8> for Pos {
    fn mul_assign(&mut self, s: i8) {
        *self = *self * s;
    }
}

impl Mul<u8> for Pos {
    type Output = Pos;

    fn mul(self, s: u8) -> Pos {
        let y = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let x = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Pos(y, x)
    }
}

impl MulAssign<u8> for Pos {
    fn mul_assign(&mut self, s: u8) {
        *self = *self * s;
    }
}

impl From<Pos> for (i8, i8) {
    fn from(c: Pos) -> Self {
        (c.0, c.1)
    }
}

impl From<Pos> for (u8, u8) {
    fn from(c: Pos) -> Self {
        ((c.0 as i16 + 128) as u8, (c.1 as i16 + 128) as u8)
    }
}

impl From<Pos> for glam::UVec2 {
    fn from(c: Pos) -> Self {
        let (y, x): (u8, u8) = c.into();
        glam::UVec2::new(x as u32, y as u32)
    }
}

impl From<Pos> for glam::IVec2 {
    fn from(c: Pos) -> Self {
        let (y, x): (i8, i8) = c.into();
        glam::IVec2::new(x as i32, y as i32)
    }
}

impl From<(i8, i8)> for Pos {
    fn from(value: (i8, i8)) -> Self {
        Pos(value.0, value.1)
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self::origin()
    }
}

impl<C: Coord> From<&C> for Pos {
    fn from(value: &C) -> Self {
        (*value).into()
    }
}
