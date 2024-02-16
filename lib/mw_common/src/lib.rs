/// Re-exports everything that I want to easily access from all MineWars crates
///
/// This includes stuff from this crate, and also important dependencies.
pub mod prelude {
    pub use anyhow::{anyhow, bail, ensure, Context, Error as AnyError, Result as AnyResult};
    #[cfg(feature = "bevy")]
    pub use bevy::prelude::*;
    #[cfg(feature = "bevy")]
    pub use bevy::utils::{HashMap, HashSet};
    #[cfg(not(feature = "bevy"))]
    pub use hashbrown::{HashMap, HashSet};
    #[cfg(all(feature = "bevy", not(feature = "net")))]
    pub use bevy::utils::{Duration, Instant};
    #[cfg(all(not(feature = "bevy"), feature = "net"))]
    pub use tokio::time::{Duration, Instant};
    #[cfg(all(not(feature = "bevy"), not(feature = "net")))]
    pub use std::time::{Duration, Instant};
    #[cfg(all(feature = "bevy", feature = "net"))]
    pub use std::time::{Duration, Instant};
    pub use num_traits;
    pub use num;
    pub use num_traits::{FromPrimitive, ToPrimitive};
    pub use num_derive::{FromPrimitive, ToPrimitive};
    pub use rand;
    pub use rand::prelude::*;
    pub use rand_pcg;
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
    pub use std::sync::Arc;
    pub use std::net::{IpAddr, SocketAddr};
    pub use std::path::{Path, PathBuf};
    pub use std::hash::Hash;
    pub use std::fmt::{Display, Debug};
    pub use std::marker::PhantomData;
    pub use thiserror::Error;
    pub use tracing::{debug, error, info, trace, warn};
    #[cfg(feature = "net")]
    pub use tokio;
    #[cfg(feature = "net")]
    pub use rustls;
    #[cfg(feature = "net")]
    pub use quinn;
    #[cfg(feature = "net")]
    pub use crate::net::prelude::*;
}

#[cfg(feature = "net")]
pub mod net;

pub mod algo;
pub mod driver;
pub mod grid;
pub mod plid;
pub mod game;
