use enum_map::{Enum, EnumMap};
use grid::{map::CompactMapCoordExt, MapData};

pub mod plid;
pub mod grid;
pub mod proto;
pub mod game;
pub mod algo;

/// Performant HashMap using AHash algorithm (not cryptographically secure)
pub type HashMap<K, V> = hashbrown::HashMap<K, V>;
/// Performant HashSet using AHash algorithm (not cryptographically secure)
pub type HashSet<T> = hashbrown::HashSet<T>;
