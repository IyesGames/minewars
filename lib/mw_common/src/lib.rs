/// Re-exports everything that I want to easily access from all MineWars crates
///
/// This includes stuff from this crate, and also important dependencies.
pub mod prelude {
    pub use anyhow::{anyhow, bail, ensure, Context, Error as AnyError, Result as AnyResult};
    #[cfg(feature = "bevy")]
    pub use bevy::utils::{Duration, HashMap, HashSet, Instant};
    #[cfg(not(feature = "bevy"))]
    pub use hashbrown::{HashMap, HashSet};
    pub use rand::prelude::*;
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
    #[cfg(not(feature = "bevy"))]
    pub use std::time::{Duration, Instant};
    pub use thiserror::Error;
    pub use tracing::{debug, error, info, trace, warn};
}

#[cfg(feature = "bevy")]
pub mod bevy;

pub mod grid;
