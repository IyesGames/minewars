//! Representation of various data in MineWars protocols

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MwDur(pub u8);

impl MwDur {
    #[inline(always)]
    pub fn as_millis(self) -> u16 {
        if self.0 < 0b10000000 {
            self.0 as u16
        } else if self.0 < 0b11000000 {
            ((self.0 & 0b00111111) as u16 + 13) * 10
        } else {
            ((self.0 & 0b00111111) as u16 + 8) * 100
        }
    }
    #[inline(always)]
    pub fn from_millis_lossy(millis: u16) -> Self {
        if millis < 128 {
            MwDur(millis as u8)
        } else if millis < 770 {
            let centis = millis / 10;
            if centis < 13 {
                MwDur(0b10000000)
            } else {
                MwDur(0b10000000 | (centis - 13) as u8)
            }
        } else if millis < 7200 {
            let decis = millis / 100;
            if decis < 8 {
                MwDur(0b11000000)
            } else {
                MwDur(0b11000000 | (decis - 8) as u8)
            }
        } else {
            MwDur(0xFF)
        }
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct MwRatio(pub u8);

impl MwRatio {
    #[inline(always)]
    pub fn new_lossy(mut num: u8, mut denum: u8) -> MwRatio {
        denum -= 1;
        while denum > 0x0F || num > 0x0F {
            num >>= 1;
            denum >>= 1;
        }
        MwRatio((num << 4) | denum)
    }
    #[inline(always)]
    pub fn num(self) -> u8 {
        (self.0 & 0xF0) >> 4
    }
    #[inline(always)]
    pub fn denum(self) -> u8 {
        (self.0 & 0x0F) + 1
    }
    #[inline(always)]
    pub fn as_f32(self) -> f32 {
        self.num() as f32 / self.denum() as f32
    }
    #[inline(always)]
    pub fn as_f64(self) -> f64 {
        self.num() as f64 / self.denum() as f64
    }
}

impl PartialEq for MwRatio {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        let an = self.num();
        let ad = self.denum();
        let bn = other.num();
        let bd = other.denum();
        let a = an as u32 * bd as u32;
        let b = bn as u32 * ad as u32;
        a == b
    }
}

impl PartialOrd for MwRatio {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let an = self.num();
        let ad = self.denum();
        let bn = other.num();
        let bd = other.denum();
        let a = an as u32 * bd as u32;
        let b = bn as u32 * ad as u32;
        a.partial_cmp(&b)
    }
}
