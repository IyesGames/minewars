use std::io::Write;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use super::*;

/// Extension trait for coordinates, providing operations for dense map layout
pub trait CompactMapCoordExt: Coord {
    type IterCoords: IntoIterator<Item=Self>;

    fn map_area(r: u8) -> usize;
    fn row_len(r: u8, y: i8) -> usize;
    fn aa_indent(y: i8) -> usize;
    fn xmin(r: u8, y: i8) -> i8;
    fn xmax(r: u8, y: i8) -> i8;
    fn index(r: u8, c: Self) -> usize;
    fn iter_coords(r: u8) -> Self::IterCoords;
}

pub enum MapAny<D> {
    Hex(MapData<Hex, D>),
    Sq(MapData<Sq, D>),
    Sqr(MapData<Sqr, D>),
}

impl<D> MapAny<D> {
    pub fn topology(&self) -> Topology {
        match self {
            MapAny::Hex(_) => Topology::Hex,
            MapAny::Sq(_) => Topology::Sq,
            MapAny::Sqr(_) => Topology::Sqr,
        }
    }

    pub fn specialize<R>(
        &self,
        f_hex: impl FnOnce(&MapData<Hex, D>) -> R,
        f_sq: impl FnOnce(&MapData<Sq, D>) -> R,
        f_sqr: impl FnOnce(&MapData<Sqr, D>) -> R,
    ) -> R {
        match self {
            MapAny::Hex(map) => f_hex(map),
            MapAny::Sq(map) => f_sq(map),
            MapAny::Sqr(map) => f_sqr(map),
        }
    }

    pub fn specialize_mut<R>(
        &mut self,
        f_hex: impl FnOnce(&mut MapData<Hex, D>) -> R,
        f_sq: impl FnOnce(&mut MapData<Sq, D>) -> R,
        f_sqr: impl FnOnce(&mut MapData<Sqr, D>) -> R,
    ) -> R {
        match self {
            MapAny::Hex(map) => f_hex(map),
            MapAny::Sq(map) => f_sq(map),
            MapAny::Sqr(map) => f_sqr(map),
        }
    }

    pub fn get_hex(&self) -> Option<&MapData<Hex, D>> {
        match self {
            MapAny::Hex(map) => Some(map),
            _ => None,
        }
    }

    pub fn get_sq(&self) -> Option<&MapData<Sq, D>> {
        match self {
            MapAny::Sq(map) => Some(map),
            _ => None,
        }
    }

    pub fn get_sqr(&self) -> Option<&MapData<Sqr, D>> {
        match self {
            MapAny::Sqr(map) => Some(map),
            _ => None,
        }
    }

    pub fn get_hex_mut(&mut self) -> Option<&mut MapData<Hex, D>> {
        match self {
            MapAny::Hex(map) => Some(map),
            _ => None,
        }
    }

    pub fn get_sq_mut(&mut self) -> Option<&mut MapData<Sq, D>> {
        match self {
            MapAny::Sq(map) => Some(map),
            _ => None,
        }
    }

    pub fn get_sqr_mut(&mut self) -> Option<&mut MapData<Sqr, D>> {
        match self {
            MapAny::Sqr(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_sqr(self) -> Result<MapData<Sqr, D>, Self> {
        match self {
            MapAny::Sqr(map) => Ok(map),
            _ => Err(self),
        }
    }

    pub fn try_unwrap<C: CompactMapCoordExt>(self) -> Result<MapData<C, D>, Self> {
        // RELIES ON `C` BEING ONE OF THE SUPPORTED TYPES
        unsafe {
            match (C::TOPOLOGY, self) {
                (Topology::Hex, MapAny::Hex(map)) => Ok(std::mem::transmute(map)),
                (Topology::Sq, MapAny::Sq(map)) => Ok(std::mem::transmute(map)),
                (Topology::Sqr, MapAny::Sqr(map)) => Ok(std::mem::transmute(map)),
                (_, MapAny::Hex(map)) => Err(MapAny::Hex(map)),
                (_, MapAny::Sq(map)) => Err(MapAny::Sq(map)),
                (_, MapAny::Sqr(map)) => Err(MapAny::Sqr(map)),
            }
        }
    }

    pub fn try_get<C: CompactMapCoordExt>(&self) -> Option<&MapData<C, D>> {
        // RELIES ON `C` BEING ONE OF THE SUPPORTED TYPES
        unsafe {
            match (C::TOPOLOGY, self) {
                (Topology::Hex, MapAny::Hex(map)) => Some(std::mem::transmute(map)),
                (Topology::Sq, MapAny::Sq(map)) => Some(std::mem::transmute(map)),
                (Topology::Sqr, MapAny::Sqr(map)) => Some(std::mem::transmute(map)),
                _ => None,
            }
        }
    }
}

impl<C: CompactMapCoordExt, D> From<MapData<C, D>> for MapAny<D> {
    fn from(map: MapData<C, D>) -> Self {
        // RELIES ON `C` BEING ONE OF THE SUPPORTED TYPES
        unsafe {
            match C::TOPOLOGY {
                Topology::Hex => MapAny::Hex(std::mem::transmute(map)),
                Topology::Sq => MapAny::Sq(std::mem::transmute(map)),
                Topology::Sqr => MapAny::Sqr(std::mem::transmute(map)),
            }
        }
    }
}

/// Map storage for a "radial" map, as a compact dense array.
///
/// `C` is the type of coordinate.
/// `D` is the data to store for each map cell.
pub struct MapData<C: CompactMapCoordExt, D> {
    size: u8,
    data: Vec<D>,
    _c: PhantomData<C>,
}

impl<C: CompactMapCoordExt, D: Clone> MapData<C, D> {
    /// Construct map with given radius
    ///
    /// All cells initialized with clones of the provided value.
    pub fn new(size: u8, init: D) -> Self {
        assert!(size <= 127);

        let len = C::map_area(size);

        let mut data = Vec::with_capacity(len);
        data.resize(len, init);

        Self {
            size,
            data,
            _c: PhantomData,
        }
    }
}

impl<C: CompactMapCoordExt, D> MapData<C, D> {
    /// Construct map with given radius
    ///
    /// All cells initialized with clones of the provided value.
    pub fn new_with<F: FnMut(C) -> D>(size: u8, mut f: F) -> Self {
        assert!(size <= 127);

        let len = C::map_area(size);

        let mut data = Vec::with_capacity(len);

        for c in C::iter_coords(size) {
            data.push(f(c));
        }

        Self {
            size,
            data,
            _c: PhantomData,
        }
    }
}

impl<C: CompactMapCoordExt, D> Index<C> for MapData<C, D> {
    type Output = D;

    fn index(&self, c: C) -> &D {
        let i = C::index(self.size, c);
        self.data.index(i)
    }
}

impl<C: CompactMapCoordExt, D> IndexMut<C> for MapData<C, D> {
    fn index_mut(&mut self, c: C) -> &mut D {
        let i = C::index(self.size, c);
        self.data.index_mut(i)
    }
}

impl<C: CompactMapCoordExt, D> MapData<C, D> {
    /// Radius of map (number of rings)
    pub fn size(&self) -> u8 {
        self.size
    }

    /// Construct new map based on data from another map
    pub fn convert<T, F: FnMut(C, &T) -> D>(
        old: &MapData<C, T>, mut f: F
    ) -> MapData<C, D> {
        Self {
            size: old.size,
            data: old.iter().map(|(c, t)| f(c, t)).collect(),
            _c: PhantomData,
        }
    }

    /// Construct new map based on partial data from another map
    pub fn convert_trim<T, F: FnMut(C, C, &T) -> D>(
        old: &MapData<C, T>, new_size: u8, offset: C, mut f: F
    ) -> Result<MapData<C, D>, OutOfBoundsError> {
        let (x, y): (i8, i8) = offset.into();

        if x.abs() as u8 + new_size > old.size || y.abs() as u8 + new_size > old.size {
            return Err(OutOfBoundsError);
        }

        Ok(Self {
            size: new_size,
            data: old.iter_at(offset, new_size).map(|(c0, c1, t)| f(c0, c1, t)).collect(),
            _c: PhantomData,
        })
    }

    /// Print map as ascii art
    ///
    /// Given closure provides byte to output for each cell
    pub fn ascii_art<W: Write, F: Fn(C, &D) -> u8>(
        &self, w: &mut W, f: F
    ) -> std::io::Result<()>
    {
        let mut y = -(self.size as i8);
        let mut next_row = 0;

        for (i, (c, d)) in self.iter().enumerate() {
            if i == next_row {
                if i != 0 {
                    w.write(&[b'\n'])?;
                }

                for _ in 0..C::aa_indent(y) {
                    w.write(&[b' '])?;
                }

                next_row += C::row_len(self.size, y);
                y += 1;
            }

            w.write(&[b' ', f(c, d)])?;
        }

        w.write(&[b'\n'])?;

        Ok(())
    }

    pub fn data(&self) -> &[D] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [D] {
        &mut self.data
    }

    pub fn iter_coords(&self, max_r: Option<u8>) -> C::IterCoords {
        let r = max_r.map(|r| r.min(self.size)).unwrap_or(self.size);
        C::iter_coords(r)
    }

    pub fn iter(&self) -> impl Iterator<Item=(C, &D)> {
        self.iter_coords(None).into_iter().zip(self.data.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=(C, &mut D)> {
        self.iter_coords(None).into_iter().zip(self.data.iter_mut())
    }

    fn iter_at(&self, offset: C, r: u8) -> impl Iterator<Item=(C, C, &D)> {
        self.iter_coords(Some(r)).into_iter().map(move |c| {
            (c, c + offset, &self[c + offset])
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::hex::Hex;

    fn rings_hex() -> MapData<Hex, u8> {
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
               0,0,0,0,
              2,2,9,0,0,
             2,0,2,1,3,3,
            0,2,2,0,3,1,3,
             0,1,0,0,3,3,
              0,1,8,0,1,
               0,1,1,1,
        ];
        let map = rings_hex();
        assert_eq!(map.data(), out);
    }

    #[test]
    fn rings_hex_trim() {
        let out = &[
              1,3,2,
             2,2,0,3,
            0,1,0,0,3,
             0,1,8,0,
              1,2,2,
        ];
        let map = rings_hex();
        let map2 = MapData::convert_trim(
            &map, 2, Hex(-1, 1),
            |new, old, d| {
                if old.1 < 0 || new.1 > 1 {
                    *d + 1
                } else {
                    *d
                }
            }
        ).unwrap();
        assert_eq!(map2.data(), out);
    }

    #[test]
    fn rings_hex_ascii() {
        let out =
r#"
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
        }).unwrap();
        assert_eq!(ascii.get_ref(), &out.as_bytes()[1..]);
    }
}
