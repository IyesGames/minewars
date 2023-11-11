use crate::prelude::*;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct TimeSpec(u8);

// impl TimeSpec {

//     pub fn from_millis(x: u16) -> Result<TimeSpec, ()> {
//         if x <= 127 {
//             Ok(x)
//         } else if x % 10 == 0 {
//             TimeSpec::from_centis(x / 10)
//         } else if x % 100 == 0 {
//             TimeSpec::from_decis(x / 100)
//         } else {
//             Err(())
//         }
//     }

//     pub fn from_millis_lossy(x: u16) -> TimeSpec {
//         if x < 128 {
//             TimeSpec(x as u8)
//         } else {
//             TimeSpec(255)
//         }
//     }

//     pub fn from_centis_lossy(mut x: u16) -> TimeSpec {
//         if x <= 12 {
//             x *= 10; // convert to millis
//             TimeSpec(x as u8)
//         } else if x < 64 + 13 {
//             x = 128 + (x - 13);
//             TimeSpec(x as u8)
//         } else if x < 720 {
//             x /= 10; // convert to decis
//             x = 128 + 64 + (x - 8);
//             TimeSpec(x as u8)
//         } else {
//             TimeSpec(255)
//         }
//     }

//     pub fn from_decis_lossy(mut x: u16) -> TimeSpec {
//         if x == 0 {
//             TimeSpec(0)
//         } else if x == 1 {
//             TimeSpec(100) // convert to millis
//         } else if x < 8 {
//             x = (x - 2) * 10; // convert to centis
//             x = 128 + (x - 13);
//             TimeSpec(x as u8)
//         } else if x < 64 + 8 {
//             x = 128 + 64 + (x - 8);
//             TimeSpec(x as u8)
//         } else {
//             TimeSpec(255)
//         }
//     }
// }

// 0..127 millis = 0..127
// 13..77 centis = 128 + 0..64 (bias -13)
// 8..72  decis = 128 + 64 + 0..64 (bias -8)
