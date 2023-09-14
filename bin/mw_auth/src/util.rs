use std::{path::Path, collections::HashSet, hash::Hash};

use rustls::{Certificate, PrivateKey};

use crate::config::ControlListMode;

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

