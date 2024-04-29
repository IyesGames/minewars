//! Types we use to encode time

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
