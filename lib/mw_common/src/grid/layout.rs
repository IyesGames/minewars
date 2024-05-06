use super::*;

pub trait IsMapKey: Copy + Sized + Send + Sync + 'static {
    type PreferredLayout: MapDataLayout<Self>;
}

impl IsMapKey for Pos {
    type PreferredLayout = MortonLayout;
}
impl IsMapKey for Hex {
    type PreferredLayout = MortonLayout;
}
impl IsMapKey for Sq {
    type PreferredLayout = MortonLayout;
}

pub trait MapDataLayout<T: IsMapKey>: Clone + Sized + Send + Sync + 'static {
    fn new(r: u8) -> Self;
    fn radius(&self) -> u8;
    fn require_size(&self) -> usize;
    fn in_bounds(&self, c: T) -> bool;
    fn index(&self, c: T) -> usize;
    fn coord_at(&self, i: usize) -> T;
}

pub trait RekeyLayout<C1: IsMapKey, C2: IsMapKey, L: MapDataLayout<C1>>: MapDataLayout<C2> {
    fn rekey_from<D>(old_layout: L, old_data: Vec<D>) -> (Self, Vec<D>);
}

#[derive(Clone)]
pub struct LinearLayout {
    r: u8,
}

impl<T: Into<Pos> + From<Pos> + IsMapKey> MapDataLayout<T> for LinearLayout {
    fn new(r: u8) -> Self {
        LinearLayout { r }
    }
    fn radius(&self) -> u8 {
        self.r
    }
    fn require_size(&self) -> usize {
        self.index(Pos(self.r as i8, self.r as i8))
    }
    fn in_bounds(&self, c: T) -> bool {
        let pos = c.into();
        let r = self.r as i8;
        pos.y() >= -r && pos.y() <= r && pos.x() >= -r && pos.x() <= r
    }
    fn index(&self, c: T) -> usize {
        let pos = c.into();
        let dim = self.r as usize * 2 + 1;
        let y = (pos.y() as i32 + self.r as i32) as usize;
        let x = (pos.x() as i32 + self.r as i32) as usize;
        y * dim + x
    }
    fn coord_at(&self, i: usize) -> T {
        let dim = self.r as usize * 2 + 1;
        let uy = i / dim;
        let ux = i % dim;
        let y = uy as i16 - self.r as i16;
        let x = ux as i16 - self.r as i16;
        let mut pos = Pos::default();
        pos.set_y(y as i8);
        pos.set_x(x as i8);
        T::from(pos)
    }
}

impl<C1, C2> RekeyLayout<C1, C2, LinearLayout> for LinearLayout
where
    C1: From<Pos> + Into<Pos> + IsMapKey,
    C2: From<Pos> + Into<Pos> + IsMapKey,
{
    fn rekey_from<D>(old_layout: LinearLayout, old_data: Vec<D>) -> (Self, Vec<D>) {
        (old_layout, old_data)
    }
}

#[derive(Clone)]
pub struct MortonLayout {
    r: u8,
}

impl<T: Into<Pos> + From<Pos> + IsMapKey> MapDataLayout<T> for MortonLayout {
    fn new(r: u8) -> Self {
        MortonLayout { r }
    }
    fn radius(&self) -> u8 {
        self.r
    }
    fn require_size(&self) -> usize {
        self.index(Pos(self.r as i8, self.r as i8))
    }
    fn in_bounds(&self, c: T) -> bool {
        let pos = c.into();
        let r = self.r as i8;
        pos.y() >= -r && pos.y() <= r && pos.x() >= -r && pos.x() <= r
    }
    fn index(&self, c: T) -> usize {
        let pos = c.into();
        let y = (pos.y() as i16 + self.r as i16) as u8;
        let x = (pos.x() as i16 + self.r as i16) as u8;
        morton_encoding::morton_encode_array([x, y])
    }
    fn coord_at(&self, i: usize) -> T {
        let [ux, uy]: [u8; 2] = morton_encoding::morton_decode_array(i);
        let y = uy as i16 - self.r as i16;
        let x = ux as i16 - self.r as i16;
        let mut pos = Pos::default();
        pos.set_y(y as i8);
        pos.set_x(x as i8);
        T::from(pos)
    }
}

impl<C1, C2> RekeyLayout<C1, C2, MortonLayout> for MortonLayout
where
    C1: From<Pos> + Into<Pos> + IsMapKey,
    C2: From<Pos> + Into<Pos> + IsMapKey,
{
    fn rekey_from<D>(old_layout: MortonLayout, old_data: Vec<D>) -> (Self, Vec<D>) {
        (old_layout, old_data)
    }
}

#[derive(Clone)]
pub struct HexDenseLayout {
    r: u8,
    map_area: usize,
    row0: usize,
}

impl MapDataLayout<Hex> for HexDenseLayout {
    fn new(r: u8) -> Self {
        HexDenseLayout {
            r,
            map_area: Hex::map_area(r),
            row0: Hex::map_area(r) / 2 - r as usize,
        }
    }
    fn radius(&self) -> u8 {
        self.r
    }
    fn require_size(&self) -> usize {
        self.map_area
    }
    fn in_bounds(&self, c: Hex) -> bool {
        let xmin = Hex::xmin(self.r, c.y());
        let xmax = Hex::xmax(self.r, c.y());
        let r = self.r as i8;
        c.y() >= -r && c.y() <= r && c.x() >= xmin && c.x() <= xmax
    }
    fn index(&self, c: Hex) -> usize {
        // PERF: this is a very naive and suboptimal implementation
        // (and likely to be a perf hotspot)

        let y = c.y() as isize;
        let x = c.x() as isize;

        let xmin = Hex::xmin(self.r, c.y()) as isize;
        let xmax = Hex::xmax(self.r, c.y()) as isize;

        let r = self.r as isize;
        let row0 = self.row0 as isize;

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
    fn coord_at(&self, i: usize) -> Hex {
        todo!()
    }
}
