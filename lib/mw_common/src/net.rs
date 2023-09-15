use crate::prelude::*;

use rustls::{Certificate, PrivateKey};

/// How to interpret a list of restrictions for security
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum ControlListMode {
    /// Everything else, except for what is in the list, is allowed
    Denylist,
    /// Only what is in the list is allowed
    Allowlist,
}

pub async fn load_cert(path: &Path) -> Result<Certificate, tokio::io::Error> {
    let bytes = tokio::fs::read(path).await?;
    Ok(Certificate(bytes))
}

pub async fn load_key(path: &Path) -> Result<PrivateKey, tokio::io::Error> {
    let bytes = tokio::fs::read(path).await?;
    Ok(PrivateKey(bytes))
}

pub fn check_list<T: Eq + Hash>(mode: ControlListMode, list: &HashSet<T>, value: &T) -> bool {
    match mode {
        ControlListMode::Denylist => {
            !list.contains(value)
        }
        ControlListMode::Allowlist => {
            list.contains(value)
        }
    }
}
