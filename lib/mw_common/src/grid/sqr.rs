use derive_more::*;

use std::ops::{Mul, MulAssign};
use std::iter::FusedIterator;

use super::pos::Pos;
use super::{Coord, map::CompactMapCoordExt, OutOfBoundsError};

/// Square Coordinate (Rhomboid rings)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Add, AddAssign, Sub, SubAssign, Neg)]
pub struct Sqr(pub i8, pub i8);

impl From<Pos> for Sqr {
    fn from(pos: Pos) -> Self {
        Sqr(pos.0, pos.1)
    }
}

impl From<Sqr> for Pos {
    fn from(sqr: Sqr) -> Self {
        Pos(sqr.0, sqr.1)
    }
}

impl Coord for Sqr {
    const N0: usize = 4;
    const N1: usize = 8;
    const N2: usize = 8;

    const TOPOLOGY: super::Topology = super::Topology::Sqr;

    type IterN0 = IterNeigh;
    type IterN1 = IterNeigh;
    type IterN2 = IterKnights;
    type IterRing = IterRing;

    fn origin() -> Self {
        Sqr(0, 0)
    }

    fn distance(self, other: Self) -> u16 {
        let dx = (self.0 as i16 - other.0 as i16).abs() as u16;
        let dy = (self.1 as i16 - other.1 as i16).abs() as u16;

        dx + dy
    }

    fn ring(self) -> u8 {
        self.0.abs() as u8 + self.1.abs() as u8
    }

    fn ring_len(r: u8) -> usize {
        r as usize * 4
    }

    fn translation(self) -> glam::Vec2 {
        glam::Vec2::new(
            self.0 as f32,
            self.1 as f32,
        )
    }

    fn from_f32_clamped((x, y): (f32, f32)) -> Self {
        let mut rx = x.round();
        let mut ry = y.round();

        ry = ry.clamp(-127.0, 127.0);
        rx = rx.clamp(-127.0, 127.0);

        Sqr(rx as i8, ry as i8)
    }

    fn iter_n0(self) -> IterNeigh {
        IterNeigh {
            c: self,
            i: 4,
        }
    }

    fn iter_n1(self) -> IterNeigh {
        IterNeigh {
            c: self,
            i: 0,
        }
    }

    fn iter_n2(self) -> IterKnights {
        IterKnights {
            c: self,
            i: 0,
        }
    }

    fn iter_ring(self, r: u8) -> IterRing {
        IterRing {
            edge: 0,
            cur: Sqr(0, -(r as i8)),
            center: self,
        }
    }
}

impl TryFrom<(f32, f32)> for Sqr {
    type Error = OutOfBoundsError;
    fn try_from((x, y): (f32, f32)) -> Result<Self, Self::Error> {
        let rx = x.round();
        let ry = y.round();

        if ry < -127.0 || ry > 127.0 || rx < -127.0 || rx > 127.0 {
            Err(OutOfBoundsError)
        } else {
            Ok(Sqr(rx as i8, ry as i8))
        }
    }
}

impl Mul<i8> for Sqr {
    type Output = Sqr;

    fn mul(self, s: i8) -> Sqr {
        let x = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let y = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Sqr(x, y)
    }
}

impl MulAssign<i8> for Sqr {
    fn mul_assign(&mut self, s: i8) {
        *self = *self * s;
    }
}

impl Mul<u8> for Sqr {
    type Output = Sqr;

    fn mul(self, s: u8) -> Sqr {
        let x = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let y = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Sqr(x, y)
    }
}

impl MulAssign<u8> for Sqr {
    fn mul_assign(&mut self, s: u8) {
        *self = *self * s;
    }
}

impl From<Sqr> for (i8, i8) {
    fn from(c: Sqr) -> Self {
        (c.0, c.1)
    }
}

impl From<Sqr> for (u8, u8) {
    fn from(c: Sqr) -> Self {
        ((c.0 as i16 + 128) as u8, (c.1 as i16 + 128) as u8)
    }
}

impl From<Sqr> for glam::UVec2 {
    fn from(c: Sqr) -> Self {
        let (x, y): (u8, u8) = c.into();
        glam::UVec2::new(x as u32, y as u32)
    }
}

impl From<Sqr> for glam::IVec2 {
    fn from(c: Sqr) -> Self {
        let (x, y): (i8, i8) = c.into();
        glam::IVec2::new(x as i32, y as i32)
    }
}

impl Default for Sqr {
    fn default() -> Self {
        Self::origin()
    }
}

impl Sqr {
    const NEIGH: [Sqr; 8] = [
        // + diagonals = N1
        Sqr(1, 1), Sqr(-1, 1), Sqr(-1, -1), Sqr(1, -1),
        // N0
        Sqr(0, -1), Sqr(1, 0), Sqr(0, 1), Sqr(-1, 0),
    ];
    const KNIGHTS: [Sqr; 8] = [
        Sqr(-1, -2), Sqr(1, -2),
        Sqr(-2, -1), Sqr(2, -1),
        Sqr(-2, 1), Sqr(2, 1),
        Sqr(-1, 2), Sqr(1, 2),
    ];
}

#[derive(Debug)]
pub struct IterNeigh {
    c: Sqr,
    i: u8,
}

impl Iterator for IterNeigh {
    type Item = Sqr;

    fn next(&mut self) -> Option<Sqr> {
        let i = self.i as usize;
        if i >= Sqr::NEIGH.len() {
            None
        } else {
            let r = Sqr::NEIGH[i] + self.c;
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
    c: Sqr,
    i: u8,
}

impl Iterator for IterKnights {
    type Item = Sqr;

    fn next(&mut self) -> Option<Sqr> {
        let i = self.i as usize;
        if i >= Sqr::KNIGHTS.len() {
            None
        } else {
            let r = Sqr::KNIGHTS[i] + self.c;
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
        Sqr::KNIGHTS.len()
    }
}

impl FusedIterator for IterKnights {}

#[derive(Debug)]
pub struct IterRing {
    edge: u8,
    cur: Sqr,
    center: Sqr,
}

impl Iterator for IterRing {
    type Item = Sqr;

    fn next(&mut self) -> Option<Sqr> {
        let r = self.cur;
        match self.edge {
            0 => {
                // top->right
                self.cur.0 += 1;
                self.cur.1 += 1;
                if self.cur.1 == 0 {
                    self.edge += 1;
                }
			}
            1 => {
                // right->bot
                self.cur.0 += -1;
                self.cur.1 += 1;
                if self.cur.0 == 0 {
                    self.edge += 1;
                }
			}
            2 => {
                // bot->left
                self.cur.0 += -1;
                self.cur.1 += -1;
                if self.cur.1 == 0 {
                    self.edge += 1;
                }
			}
            3 => {
                // left->top
                self.cur.0 += 1;
                self.cur.1 += -1;
                if self.cur.0 == 0 {
                    self.edge += 1;
                }
			}
			_ => return None,
        }
        Some(r + self.center)
    }
}

impl FusedIterator for IterRing {}

impl CompactMapCoordExt for Sqr {
    type IterCoords = IterCoords;

    fn map_area(r: u8) -> usize {
        let r = r as usize;
        r * r * 2 + (r * 2 + 1)
    }
    fn xmin(r: u8, _y: i8) -> i8 {
        -(r as i8)
    }
    fn xmax(r: u8, _y: i8) -> i8 {
        r as i8
    }
    fn index(r: u8, c: Self) -> usize {
        let r = r as usize;
        let x = c.0 as isize;
        let y = c.1 as isize;

        assert!(y.abs() as usize <= r);

        // PERF: improve math and reduce branches

        if y == -(r as isize) {
            0
        } else if y <= 0 {
            let ny = r + y as usize;
            (1 + ((ny - 1) * 2 + 1)) * ny / 2 + (x + ny as isize) as usize
        } else {
            let ny = r - y as usize;
            Self::map_area(r as u8) - (1 + (ny * 2 + 1)) * (ny + 1) / 2 + (x + ny as isize) as usize
        }
    }
    fn row_len(r: u8, y: i8) -> usize {
        (r as usize * 2 + 1) - y.abs() as usize * 2
    }
    fn aa_indent(y: i8) -> usize {
        y.abs() as usize * 2
    }
    fn iter_coords(r: u8) -> Self::IterCoords {
        IterCoords {
            r,
            next: Some(Sqr(0, -(r as i8))),
        }
    }
}

pub struct IterCoords {
    r: u8,
    next: Option<Sqr>,
}

impl Iterator for IterCoords {
    type Item = Sqr;

    fn next(&mut self) -> Option<Sqr> {
        let r = self.r as i8;

        let next = self.next;

        if let Some(Sqr(y, x)) = &mut self.next {
            let xmax = (r - y.abs()) * 2 + 1;

            *x += 1;

            if *x > xmax {
                *y += 1;
                *x = -(r - y.abs()) * 2 + 1;
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
        Sqr::map_area(self.r)
    }
}

impl FusedIterator for IterCoords {}

