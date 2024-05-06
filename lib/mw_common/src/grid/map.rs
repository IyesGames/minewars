use std::io::Write;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use super::*;
use super::layout::*;

pub type MapDataPos<D> = MapData<Pos, D, <Pos as IsMapKey>::PreferredLayout>;
pub type MapDataC<C, D> = MapData<C, D, <C as IsMapKey>::PreferredLayout>;

/// Map storage for a "radial" map
///
/// `C` is the type of coordinate.
/// `D` is the data to store for each map cell.
#[derive(Clone)]
pub struct MapData<C: IsMapKey, D, L: MapDataLayout<C>> {
    layout: L,
    data: Vec<D>,
    _c: PhantomData<C>,
}

impl<C: IsMapKey, D: Clone, L: MapDataLayout<C>> MapData<C, D, L> {
    /// Construct map with given radius
    ///
    /// All cells initialized with clones of the provided value.
    pub fn new(size: u8, init: D) -> Self {
        assert!(size <= 127);
        let layout = L::new(size);
        let len = layout.require_size();

        let mut data = Vec::with_capacity(len);
        data.resize(len, init);

        Self {
            layout,
            data,
            _c: PhantomData,
        }
    }
}

impl<C: IsMapKey, D, L: MapDataLayout<C>> MapData<C, D, L> {
    /// Construct map with given radius
    ///
    /// All cells initialized with clones of the provided value.
    pub fn new_with<F: FnMut(C) -> D>(size: u8, mut f: F) -> Self {
        assert!(size <= 127);
        let layout = L::new(size);
        let len = layout.require_size();

        let mut data = Vec::with_capacity(len);

        for i in 0..len {
            data.push(f(layout.coord_at(i)));
        }

        Self {
            layout,
            data,
            _c: PhantomData,
        }
    }
}

impl<C: IsMapKey, D, L: MapDataLayout<C>> Index<C> for MapData<C, D, L> {
    type Output = D;

    fn index(&self, c: C) -> &D {
        let i = self.layout.index(c);
        self.data.index(i)
    }
}

impl<C: IsMapKey, D, L: MapDataLayout<C>> IndexMut<C> for MapData<C, D, L> {
    fn index_mut(&mut self, c: C) -> &mut D {
        let i = self.layout.index(c);
        self.data.index_mut(i)
    }
}

impl<C: IsMapKey, D, L: MapDataLayout<C>> MapData<C, D, L> {
    /// Radius of map (number of rings)
    pub fn size(&self) -> u8 {
        self.layout.radius()
    }

    /// Construct new map based on data from another map
    pub fn convert<T, F: FnMut(C, &D) -> T>(&self, mut f: F) -> MapData<C, T, L> {
        MapData::<C, T, L> {
            layout: self.layout.clone(),
            data: self.iter().map(|(c, d)| f(c, d)).collect(),
            _c: PhantomData,
        }
    }

    pub fn rekey<C2, L2>(self) -> MapData<C2, D, L2>
    where
        C2: IsMapKey,
        L2: RekeyLayout<C, C2, L>,
    {
        let (layout, data) = L2::rekey_from(self.layout, self.data);
        MapData {
            layout,
            data,
            _c: PhantomData,
        }
    }

    pub fn get(&self, c: C) -> Option<&D> {
        if !self.layout.in_bounds(c) {
            return None;
        }
        let i = self.layout.index(c);
        self.data.get(i)
    }

    pub fn get_mut(&mut self, c: C) -> Option<&mut D> {
        if !self.layout.in_bounds(c) {
            return None;
        }
        let i = self.layout.index(c);
        self.data.get_mut(i)
    }

    pub fn data(&self) -> &[D] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [D] {
        &mut self.data
    }

    pub fn iter_data(&self) -> impl Iterator<Item = &D> {
        self.data.iter()
    }

    pub fn iter_data_mut(&mut self) -> impl Iterator<Item = &mut D> {
        self.data.iter_mut()
    }

    pub fn iter(&self) -> impl Iterator<Item = (C, &D)> {
        self.data.iter().enumerate()
            .map(|(i, d)| (self.layout.coord_at(i), d))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (C, &mut D)> {
        let data = &mut self.data;
        let layout = &self.layout;
        data.iter_mut().enumerate()
            .map(|(i, d)| (layout.coord_at(i), d))
    }
}

impl<C: Coord, D, L: MapDataLayout<C>> MapData<C, D, L> {
    /// Construct new map based on partial data from another map
    pub fn convert_trim<T, F: FnMut(C, C, &D) -> T>(
        &self,
        new_size: u8,
        offset: C,
        mut f: F,
    ) -> Result<MapData<C, T, L>, OutOfBoundsError> {
        let (x, y): (i8, i8) = offset.into();

        if x.abs() as u8 + new_size > self.size() || y.abs() as u8 + new_size > self.size() {
            return Err(OutOfBoundsError);
        }

        Ok(MapData::<C, T, L> {
            layout: self.layout.clone(),
            data: self
                .iter_at(offset, new_size)
                .map(|(c0, c1, d)| f(c0, c1, d))
                .collect(),
            _c: PhantomData,
        })
    }

    /// Print map as ascii art
    ///
    /// Given closure provides byte to output for each cell
    pub fn ascii_art<W: Write, F: Fn(C, &D) -> u8>(&self, w: &mut W, f: F) -> std::io::Result<()> {
        let mut y = -(self.size() as i8);
        let mut next_row = 0;

        for (i, (c, d)) in self.iter().enumerate() {
            if i == next_row {
                if i != 0 {
                    w.write(&[b'\n'])?;
                }

                for _ in 0..C::aa_indent(y) {
                    w.write(&[b' '])?;
                }

                next_row += C::row_len(self.size(), y);
                y += 1;
            }

            w.write(&[b' ', f(c, d)])?;
        }

        w.write(&[b'\n'])?;

        Ok(())
    }

    pub fn iter_coords(&self, max_r: Option<u8>) -> C::IterCoords {
        let r = max_r.map(|r| r.min(self.size())).unwrap_or(self.size());
        C::iter_coords(r)
    }

    fn iter_at(&self, offset: C, r: u8) -> impl Iterator<Item = (C, C, &D)> {
        self.iter_coords(Some(r))
            .into_iter()
            .map(move |c| (c, c + offset, &self[c + offset]))
    }
}

impl<C: IsMapKey, D, L: MapDataLayout<C>> MapData<C, D, L> {
    pub fn get_ringmask<C2: Coord + Into<C>>(&self, c: C2, mut f: impl FnMut(&D) -> bool) -> u8 {
        let mut r = 0;
        for c2 in c.iter_n1() {
            r = r << 1;
            if let Some(d) = self.get(c2.into()) {
                if f(d) {
                    r |= 1;
                }
            }
        }
        r
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::grid::hex::Hex;

    fn rings_hex() -> MapDataC<Hex, u8> {
        let mut map = MapData::new(3, 0);
        for c in Hex(0, 1).iter_ring(2) {
            map[c] = 1;
        }
        for c in Hex(-1, -1).iter_ring(1) {
            map[c] = 2;
        }
        for c in Hex(2, 0).iter_ring(1) {
            map[c] = 3;
        }
        map[Hex(-1, 2)] = 8;
        map[Hex(1, -2)] = 9;
        map
    }

    #[test]
    fn rings_hex_check() {
        let out = &[
            0, 0, 0, 0, 2, 2, 9, 0, 0, 2, 0, 2, 1, 3, 3, 0, 2, 2, 0, 3, 1, 3, 0, 1, 0, 0, 3, 3, 0,
            1, 8, 0, 1, 0, 1, 1, 1,
        ];
        let map = rings_hex();
        assert_eq!(map.data(), out);
    }

    #[test]
    fn rings_hex_trim() {
        let out = &[1, 3, 2, 2, 2, 0, 3, 0, 1, 0, 0, 3, 0, 1, 8, 0, 1, 2, 2];
        let map = rings_hex();
        let map2 = MapData::convert_trim(&map, 2, Hex(-1, 1), |new, old, d| {
            if old.1 < 0 || new.1 > 1 {
                *d + 1
            } else {
                *d
            }
        })
        .unwrap();
        assert_eq!(map2.data(), out);
    }

    #[test]
    fn rings_hex_ascii() {
        let out = r#"
    o O o o
   Y y B o o
  y O y X z z
 o y Y o Z x z
  o x O o Z z
   o x A o X
    o x X x
"#;
        let map = rings_hex();
        let mut ascii = std::io::Cursor::new(Vec::new());
        map.ascii_art(&mut ascii, |c, d| {
            if c.0.abs() == 1 {
                match d {
                    0 => b'O',
                    1 => b'X',
                    2 => b'Y',
                    3 => b'Z',
                    8 => b'A',
                    9 => b'B',
                    _ => b'?',
                }
            } else {
                match d {
                    0 => b'o',
                    1 => b'x',
                    2 => b'y',
                    3 => b'z',
                    _ => b'?',
                }
            }
        })
        .unwrap();
        assert_eq!(ascii.get_ref(), &out.as_bytes()[1..]);
    }
}
