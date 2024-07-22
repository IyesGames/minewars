#![feature(trait_upcasting)]

pub mod prelude {
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
    pub use std::marker::PhantomData;
    pub use std::sync::Arc;
    pub use std::hash::Hash;
    pub use std::fmt::{Display, Debug};
    pub use std::path::{Path, PathBuf};
    pub use bevy::utils::{Duration, Instant};
    pub use bevy::utils::{HashMap, HashSet};
    pub use bevy::prelude::*;
    pub use anyhow::{anyhow, bail, ensure, Context, Error as AnyError, Result as AnyResult};
    pub use iyes_bevy_extras::prelude::*;
    pub use crate::settings_manager::prelude::*;
}

pub mod settings_manager;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::settings_manager::plugin,
    ));
}
