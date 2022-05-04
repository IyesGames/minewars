use derive_more::*;

use std::ops::{Mul, MulAssign};

/// Topology-agnostic (type erased) coordinate
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Add, AddAssign, Sub, SubAssign, Neg)]
pub struct Pos(pub i8, pub i8);

impl Pos {
    pub fn origin() -> Self {
        Pos(0, 0)
    }
}

impl Mul<i8> for Pos {
    type Output = Pos;

    fn mul(self, s: i8) -> Pos {
        let x = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let y = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Pos(x, y)
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
        let x = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let y = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Pos(x, y)
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
        let (x, y): (u8, u8) = c.into();
        glam::UVec2::new(x as u32, y as u32)
    }
}

impl From<Pos> for glam::IVec2 {
    fn from(c: Pos) -> Self {
        let (x, y): (i8, i8) = c.into();
        glam::IVec2::new(x as i32, y as i32)
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self::origin()
    }
}
