use derive_more::*;

use std::iter::FusedIterator;
use std::ops::{Mul, MulAssign};

use super::pos::Pos;
use super::{Coord, OutOfBoundsError};

/// Square Coordinate (square rings)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Add, AddAssign, Sub, SubAssign, Neg,
)]
pub struct Sq(pub i8, pub i8);

impl From<Pos> for Sq {
    fn from(pos: Pos) -> Self {
        Sq(pos.0, pos.1)
    }
}

impl From<Sq> for Pos {
    fn from(sq: Sq) -> Self {
        Pos(sq.0, sq.1)
    }
}

impl Coord for Sq {
    const N0: usize = 4;
    const N1: usize = 8;
    const N2: usize = 8;

    const TOPOLOGY: super::Topology = super::Topology::Sq;

    type IterN0 = IterNeigh;
    type IterN1 = IterNeigh;
    type IterN2 = IterKnights;
    type IterRing = IterRing;
    type IterCoords = IterCoords;

    fn origin() -> Self {
        Sq(0, 0)
    }

    fn distance(self, other: Self) -> u16 {
        let dy = (self.0 as i16 - other.0 as i16).abs() as u16;
        let dx = (self.1 as i16 - other.1 as i16).abs() as u16;

        dx.max(dy)
    }

    fn ring(self) -> u8 {
        (self.0.abs() as u8).max(self.1.abs() as u8)
    }

    fn ring_len(r: u8) -> usize {
        (r as usize * 2 + 1) * 4
    }

    fn translation(self) -> glam::Vec2 {
        glam::Vec2::new(self.1 as f32, self.0 as f32)
    }

    fn from_f32_clamped((x, y): (f32, f32)) -> Self {
        let mut rx = x.round();
        let mut ry = y.round();

        ry = ry.clamp(-127.0, 127.0);
        rx = rx.clamp(-127.0, 127.0);

        Sq(ry as i8, rx as i8)
    }

    fn iter_n0(self) -> IterNeigh {
        IterNeigh { c: self, i: 4 }
    }

    fn iter_n1(self) -> IterNeigh {
        IterNeigh { c: self, i: 0 }
    }

    fn iter_n2(self) -> IterKnights {
        IterKnights { c: self, i: 0 }
    }

    fn iter_ring(self, r: u8) -> IterRing {
        IterRing {
            edge: 0,
            radius: r as i8,
            cur: Sq(-(r as i8), -(r as i8)),
            center: self,
        }
    }

    fn map_area(r: u8) -> usize {
        let r = r as usize;
        let r2 = r * 2 + 1;
        r2 * r2
    }
    fn xmin(r: u8, _y: i8) -> i8 {
        -(r as i8)
    }
    fn xmax(r: u8, _y: i8) -> i8 {
        r as i8
    }
    fn index(r: u8, c: Self) -> usize {
        assert!(r <= 127);
        let r = r as i8;
        assert!(c.0 >= -r && c.0 <= r && c.1 >= -r && c.1 <= r);
        let r = r as i16;
        let w = r * 2 + 1;
        let x = c.1 as i16 + r;
        let y = c.0 as i16 + r;
        (y * w + x) as usize
    }
    fn row_len(r: u8, _y: i8) -> usize {
        r as usize * 2 + 1
    }
    fn aa_indent(_y: i8) -> usize {
        1
    }
    fn iter_coords(r: u8) -> Self::IterCoords {
        let r = r as i8;
        IterCoords {
            r,
            next: Some(Sq(-r, -r)),
        }
    }
}

impl TryFrom<(f32, f32)> for Sq {
    type Error = OutOfBoundsError;
    fn try_from((y, x): (f32, f32)) -> Result<Self, Self::Error> {
        let rx = x.round();
        let ry = y.round();

        if ry < -127.0 || ry > 127.0 || rx < -127.0 || rx > 127.0 {
            Err(OutOfBoundsError)
        } else {
            Ok(Sq(ry as i8, rx as i8))
        }
    }
}

impl Mul<i8> for Sq {
    type Output = Sq;

    fn mul(self, s: i8) -> Sq {
        let y = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let x = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Sq(y, x)
    }
}

impl MulAssign<i8> for Sq {
    fn mul_assign(&mut self, s: i8) {
        *self = *self * s;
    }
}

impl Mul<u8> for Sq {
    type Output = Sq;

    fn mul(self, s: u8) -> Sq {
        let y = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let x = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Sq(y, x)
    }
}

impl MulAssign<u8> for Sq {
    fn mul_assign(&mut self, s: u8) {
        *self = *self * s;
    }
}

impl From<Sq> for (i8, i8) {
    fn from(c: Sq) -> Self {
        (c.0, c.1)
    }
}

impl From<Sq> for (u8, u8) {
    fn from(c: Sq) -> Self {
        ((c.0 as i16 + 128) as u8, (c.1 as i16 + 128) as u8)
    }
}

impl From<Sq> for glam::UVec2 {
    fn from(c: Sq) -> Self {
        let (y, x): (u8, u8) = c.into();
        glam::UVec2::new(x as u32, y as u32)
    }
}

impl From<Sq> for glam::IVec2 {
    fn from(c: Sq) -> Self {
        let (y, x): (i8, i8) = c.into();
        glam::IVec2::new(x as i32, y as i32)
    }
}

impl Default for Sq {
    fn default() -> Self {
        Self::origin()
    }
}

impl Sq {
    const NEIGH: [Sq; 8] = [
        // + diagonals = N1
        Sq(-1, -1),
        Sq(1, -1),
        Sq(-1, 1),
        Sq(1, 1),
        // N0
        Sq(0, 1),
        Sq(-1, 0),
        Sq(1, 0),
        Sq(0, -1),
    ];
    const KNIGHTS: [Sq; 8] = [
        Sq(-1, -2),
        Sq(1, -2),
        Sq(-2, -1),
        Sq(2, -1),
        Sq(-2, 1),
        Sq(2, 1),
        Sq(-1, 2),
        Sq(1, 2),
    ];
}

#[derive(Debug)]
pub struct IterNeigh {
    c: Sq,
    i: u8,
}

impl Iterator for IterNeigh {
    type Item = Sq;

    fn next(&mut self) -> Option<Sq> {
        let i = self.i as usize;
        if i >= Sq::NEIGH.len() {
            None
        } else {
            let r = Sq::NEIGH[i] + self.c;
            self.i += 1;
            Some(r)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (4, Some(8))
    }
}

impl FusedIterator for IterNeigh {}

#[derive(Debug)]
pub struct IterKnights {
    c: Sq,
    i: u8,
}

impl Iterator for IterKnights {
    type Item = Sq;

    fn next(&mut self) -> Option<Sq> {
        let i = self.i as usize;
        if i >= Sq::KNIGHTS.len() {
            None
        } else {
            let r = Sq::KNIGHTS[i] + self.c;
            self.i += 1;
            Some(r)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for IterKnights {
    fn len(&self) -> usize {
        Sq::KNIGHTS.len()
    }
}

impl FusedIterator for IterKnights {}

#[derive(Debug)]
pub struct IterRing {
    edge: u8,
    cur: Sq,
    radius: i8,
    center: Sq,
}

impl Iterator for IterRing {
    type Item = Sq;

    fn next(&mut self) -> Option<Sq> {
        let r = self.cur;
        match self.edge {
            0 => {
                // bot->right
                self.cur.0 += 1;
                if self.cur.0 == self.radius {
                    self.edge += 1;
                }
            }
            1 => {
                // right->top
                self.cur.1 += 1;
                if self.cur.1 == self.radius {
                    self.edge += 1;
                }
            }
            2 => {
                // top->left
                self.cur.0 += -1;
                if self.cur.0 == -self.radius {
                    self.edge += 1;
                }
            }
            3 => {
                // left->bot
                self.cur.1 += -1;
                if self.cur.1 == -self.radius {
                    self.edge += 1;
                }
            }
            _ => return None,
        }
        Some(r + self.center)
    }
}

impl FusedIterator for IterRing {}

pub struct IterCoords {
    r: i8,
    next: Option<Sq>,
}

impl Iterator for IterCoords {
    type Item = Sq;

    fn next(&mut self) -> Option<Sq> {
        let r = self.r as i8;

        let next = self.next;

        if let Some(Sq(y, x)) = &mut self.next {
            *x += 1;

            if *x > self.r {
                *y += 1;
                *x = -r;
            }

            if *y > r {
                self.next = None;
            }
        }

        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for IterCoords {
    fn len(&self) -> usize {
        Sq::map_area(self.r as u8)
    }
}

impl FusedIterator for IterCoords {}
