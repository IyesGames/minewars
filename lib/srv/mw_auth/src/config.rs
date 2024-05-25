use crate::prelude::*;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Config {
    // TODO
}

impl Config {
    /// Check for any CLI Args that override config options and modify the config accordingly.
    pub fn apply_cli(&mut self, args: &crate::cli::Args) {
    }
}
