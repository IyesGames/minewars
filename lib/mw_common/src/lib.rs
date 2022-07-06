pub mod plid;
pub mod grid;
pub mod proto;
pub mod game;
pub mod algo;
#[cfg(feature = "bevy")]
pub mod app;
#[cfg(feature = "bevy")]
pub mod host;

/// Performant HashMap using AHash algorithm (not cryptographically secure)
pub type HashMap<K, V> = hashbrown::HashMap<K, V>;
/// Performant HashSet using AHash algorithm (not cryptographically secure)
pub type HashSet<T> = hashbrown::HashSet<T>;
