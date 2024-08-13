#![feature(str_split_whitespace_remainder)]
#![feature(round_char_boundary)]

pub mod header;
pub mod map;
pub mod msg;

pub mod read;
pub mod write;

pub const FORMAT_VERSION: header::Version = header::Version(0, 0, 1 ,0);
