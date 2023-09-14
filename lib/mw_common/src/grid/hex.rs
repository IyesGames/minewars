use derive_more::*;

use std::iter::FusedIterator;
use std::ops::{Mul, MulAssign};

use super::pos::Pos;
use super::{Coord, OutOfBoundsError};

/// Axial Hex coordinate
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Add, AddAssign, Sub, SubAssign, Neg,
)]
pub struct Hex(pub i8, pub i8);

impl From<Pos> for Hex {
    fn from(pos: Pos) -> Self {
        Hex(pos.0, pos.1)
    }
}

impl From<Hex> for Pos {
    fn from(hex: Hex) -> Self {
        Pos(hex.0, hex.1)
    }
}

impl Coord for Hex {
    const N0: usize = 6;
    const N1: usize = 6;
    const N2: usize = 6;

    const TOPOLOGY: super::Topology = super::Topology::Hex;

    type IterN0 = IterNeigh;
    type IterN1 = IterNeigh;
    type IterN2 = IterDiag;
    type IterRing = IterRing;
    type IterCoords = IterCoords;

    fn origin() -> Self {
        Hex(0, 0)
    }

    fn distance(self, other: Self) -> u16 {
        let a0 = self.0;
        let a1 = self.1;
        let a2 = 0 - self.0 - self.1;
        let b0 = other.0;
        let b1 = other.1;
        let b2 = 0 - other.0 - other.1;

        let d0 = (a0 as i16 - b0 as i16).abs() as u16;
        let d1 = (a1 as i16 - b1 as i16).abs() as u16;
        let d2 = (a2 as i16 - b2 as i16).abs() as u16;

        (d0 + d1 + d2) / 2
    }

    fn ring(self) -> u8 {
        let v0 = self.0 as i16;
        let v1 = self.1 as i16;
        let v2 = 0 - v0 - v1;

        ((v0.abs() + v1.abs() + v2.abs()) / 2) as u8
    }

    fn ring_len(r: u8) -> usize {
        r as usize * 6
    }

    fn translation(self) -> glam::Vec2 {
        let y = self.0 as f32;
        let x = self.1 as f32;
        glam::Vec2::new(y * 0.5 + x, y * 0.75)
    }

    fn from_f32_clamped((y, x): (f32, f32)) -> Self {
        let z = -x - y;

        let mut rx = x.round();
        let mut ry = y.round();
        let rz = z.round();

        let dx = (rx - x).abs();
        let dy = (ry - y).abs();
        let dz = (rz - z).abs();

        if dx > dy && dx > dz {
            rx = -ry - rz;
        } else if dy > dz {
            ry = -rx - rz;
        }

        ry = ry.clamp(-127.0, 127.0);
        rx = rx.clamp(-127.0, 127.0);

        Hex(ry as i8, rx as i8)
    }

    fn iter_n0(self) -> IterNeigh {
        IterNeigh { c: self, i: 0 }
    }

    fn iter_n1(self) -> IterNeigh {
        self.iter_n0()
    }

    fn iter_n2(self) -> IterDiag {
        IterDiag { c: self, i: 0 }
    }

    fn iter_ring(self, r: u8) -> IterRing {
        IterRing {
            edge: 0,
            i: 0,
            r,
            cur: Self::RING[4] * r as i8 + self,
        }
    }

    fn map_area(r: u8) -> usize {
        // arithmetic sequence sum = total # of cells
        (12 + (r as usize - 1) * 6) * r as usize / 2 + 1
    }
    fn xmin(r: u8, y: i8) -> i8 {
        let r = r as i8;
        if y < 0 {
            -r - y
        } else {
            -r
        }
    }
    fn xmax(r: u8, y: i8) -> i8 {
        let r = r as i8;
        if y < 0 {
            r
        } else {
            r - y
        }
    }
    fn index(r: u8, c: Self) -> usize {
        // PERF: this is a very naive and suboptimal implementation
        // (and likely to be a perf hotspot)

        let y = c.0 as isize;
        let x = c.1 as isize;

        let row0 = Hex::map_area(r) as isize / 2 - r as isize;

        let xmin = Self::xmin(r, c.1) as isize;
        let xmax = Self::xmax(r, c.1) as isize;

        let r = r as isize;

        if y < 0 {
            assert!(x >= xmin);
            assert!(x <= xmax);

            let fix = -y * (r * 2 + 1);
            let miss = (1 + -y) * -y / 2;

            let xoff = x + r + y;

            let i = row0 - (fix - miss) + xoff;

            i as usize
        } else {
            assert!(x >= xmin);
            assert!(x <= xmax);

            let fix = y * (r * 2 + 1);
            let miss = y * (y - 1) / 2;

            let xoff = x + r;

            let i = row0 + (fix - miss) + xoff;

            i as usize
        }
    }
    fn row_len(r: u8, y: i8) -> usize {
        (r as usize * 2 + 1) - y.abs() as usize
    }
    fn aa_indent(y: i8) -> usize {
        y.abs() as usize
    }
    fn iter_coords(r: u8) -> Self::IterCoords {
        IterCoords {
            r,
            next: Some(Hex(-(r as i8), 0)),
        }
    }
}

impl TryFrom<(f32, f32)> for Hex {
    type Error = OutOfBoundsError;
    fn try_from((y, x): (f32, f32)) -> Result<Self, Self::Error> {
        let z = -x - y;

        let mut rx = x.round();
        let mut ry = y.round();
        let rz = z.round();

        let dx = (rx - x).abs();
        let dy = (ry - y).abs();
        let dz = (rz - z).abs();

        if dx > dy && dx > dz {
            rx = -ry - rz;
        } else if dy > dz {
            ry = -rx - rz;
        }

        if ry < -127.0 || ry > 127.0 || rx < -127.0 || rx > 127.0 {
            Err(OutOfBoundsError)
        } else {
            Ok(Hex(ry as i8, rx as i8))
        }
    }
}

impl Mul<i8> for Hex {
    type Output = Hex;

    fn mul(self, s: i8) -> Hex {
        let y = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let x = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Hex(y, x)
    }
}

impl MulAssign<i8> for Hex {
    fn mul_assign(&mut self, s: i8) {
        *self = *self * s;
    }
}

impl Mul<u8> for Hex {
    type Output = Hex;

    fn mul(self, s: u8) -> Hex {
        let y = (self.0 as i16 * s as i16).max(-128).min(127) as i8;
        let x = (self.1 as i16 * s as i16).max(-128).min(127) as i8;
        Hex(y, x)
    }
}

impl MulAssign<u8> for Hex {
    fn mul_assign(&mut self, s: u8) {
        *self = *self * s;
    }
}

impl From<Hex> for (i8, i8) {
    fn from(c: Hex) -> Self {
        (c.0, c.1)
    }
}

impl From<Hex> for (u8, u8) {
    fn from(c: Hex) -> Self {
        ((c.0 as i16 + 128) as u8, (c.1 as i16 + 128) as u8)
    }
}

impl From<Hex> for glam::UVec2 {
    fn from(c: Hex) -> Self {
        let (y, x): (u8, u8) = c.into();
        glam::UVec2::new(x as u32, y as u32)
    }
}

impl From<Hex> for glam::IVec2 {
    fn from(c: Hex) -> Self {
        let (y, x): (i8, i8) = c.into();
        glam::IVec2::new(x as i32, y as i32)
    }
}

impl Default for Hex {
    fn default() -> Self {
        Self::origin()
    }
}

impl Hex {
    const RING: [Hex; 6] = [
        Hex(-1, 0),
        Hex(-1, 1),
        Hex(0, 1),
        Hex(1, 0),
        Hex(1, -1),
        Hex(0, -1),
    ];
    const DIAG: [Hex; 6] = [
        Hex(-1, 21),
        Hex(1, 1),
        Hex(2, -1),
        Hex(1, -2),
        Hex(-1, -1),
        Hex(-2, 1),
    ];
}

#[derive(Debug)]
pub struct IterNeigh {
    c: Hex,
    i: u8,
}

impl Iterator for IterNeigh {
    type Item = Hex;

    fn next(&mut self) -> Option<Hex> {
        let i = self.i as usize;
        if i >= Hex::RING.len() {
            None
        } else {
            let r = Hex::RING[i] + self.c;
            self.i += 1;
            Some(r)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for IterNeigh {
    fn len(&self) -> usize {
        Hex::RING.len()
    }
}

impl FusedIterator for IterNeigh {}

#[derive(Debug)]
pub struct IterDiag {
    c: Hex,
    i: u8,
}

impl Iterator for IterDiag {
    type Item = Hex;

    fn next(&mut self) -> Option<Hex> {
        let i = self.i as usize;
        if i >= Hex::DIAG.len() {
            None
        } else {
            let r = Hex::DIAG[i] + self.c;
            self.i += 1;
            Some(r)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for IterDiag {
    fn len(&self) -> usize {
        Hex::DIAG.len()
    }
}

impl FusedIterator for IterDiag {}

#[derive(Debug)]
pub struct IterRing {
    edge: u8,
    i: u8,
    r: u8,
    cur: Hex,
}

impl Iterator for IterRing {
    type Item = Hex;

    fn next(&mut self) -> Option<Hex> {
        if self.edge as usize >= Hex::RING.len() {
            return None;
        }

        let dir = Hex::RING[self.edge as usize];

        let r = self.cur;

        self.cur += dir;
        self.i += 1;

        if self.i >= self.r {
            self.i = 0;
            self.edge += 1;
        }

        Some(r)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for IterRing {
    fn len(&self) -> usize {
        Hex::RING.len() * self.r as usize
    }
}

impl FusedIterator for IterRing {}

pub struct IterCoords {
    r: u8,
    next: Option<Hex>,
}

impl Iterator for IterCoords {
    type Item = Hex;

    fn next(&mut self) -> Option<Hex> {
        let r = self.r as i8;

        let next = self.next;

        if let Some(Hex(y, x)) = &mut self.next {
            let xmax = if *y < 0 { r } else { r - *y };

            *x += 1;

            if *x > xmax {
                *y += 1;
                *x = if *y < 0 { -r - *y } else { -r }
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
        Hex::map_area(self.r)
    }
}

impl FusedIterator for IterCoords {}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn neighs() {
        let mut n0: HashSet<_> = Hex::RING.iter().map(|c| *c + Hex(-3, 6)).collect();
        let mut n1: HashSet<_> = Hex::RING.iter().map(|c| *c + Hex(6, -7)).collect();
        let mut n2: HashSet<_> = Hex::DIAG.iter().map(|c| *c + Hex(-8, 5)).collect();

        // iterators return the sets in no particular order
        for c in Hex(-3, 6).iter_n0() {
            assert!(n0.remove(&c));
        }
        for c in Hex(6, -7).iter_n1() {
            assert!(n1.remove(&c));
        }
        for c in Hex(-8, 5).iter_n2() {
            assert!(n2.remove(&c));
        }
        assert!(n0.is_empty());
        assert!(n1.is_empty());
        assert!(n2.is_empty());
    }

    #[test]
    fn index() {
        assert_eq!(Hex::index(3, Hex(-3, 0)), 0);
        assert_eq!(Hex::index(3, Hex(-2, 3)), Hex::row_len(3, -3) + (1 + 3));
        assert_eq!(
            Hex::index(3, Hex(0, 2)),
            Hex::row_len(3, -3) + Hex::row_len(3, -2) + Hex::row_len(3, -1) + (3 + 2)
        );
        assert_eq!(
            Hex::index(3, Hex(2, -1)),
            Hex::row_len(3, -3)
                + Hex::row_len(3, -2)
                + Hex::row_len(3, -1)
                + Hex::row_len(3, 0)
                + Hex::row_len(3, 1)
                + 2
        );
        assert_eq!(Hex::index(3, Hex(3, 0)), Hex::map_area(3) - 1);
    }
}
