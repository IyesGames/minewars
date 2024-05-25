use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::{error::Error, fmt::{Display, Debug}};
use thiserror::Error;

pub mod methods {
    pub mod reload_config;
    pub mod create_session;
    pub mod kill_session;
    pub mod expect_player;
}

pub trait RpcMethod: Serialize + DeserializeOwned {
    const NAME: RpcMethodName;
    type Error: Error + Display + Debug + Serialize + DeserializeOwned;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum RpcMethodName {
    ReloadConfig,
    CreateSession,
    KillSession,
    ExpectPlayer,
}

#[derive(Debug, Clone, Error)]
#[derive(Serialize, Deserialize)]
pub enum RpcError {
    #[error("Invalid request.")]
    Invalid,
    #[error("Operation unsupported or disabled.")]
    Unsupported,
    #[error("Operation not permitted.")]
    Forbidden,
    /// Append the method's error type after this
    #[error("Operation failed.")]
    Method,
}

#[derive(Debug, Error)]
pub enum ResponseError<M: RpcMethod> {
    #[error("Invalid RON data: {0}")]
    RonSpanned(#[from] ron::error::SpannedError),
    #[error("Invalid RON data: {0}")]
    Ron(#[from] ron::Error),
    #[error("RPC error: {0}")]
    Rpc(RpcError),
    #[error("RPC method error: {0}")]
    Method(M::Error)
}

pub fn ser_request<M: RpcMethod>(buf: &mut Vec<u8>, request: &M) -> Result<(), ron::Error> {
    let mut ser = ron::ser::Serializer::new(buf, None)?;
    M::NAME.serialize(&mut ser)?;
    request.serialize(&mut ser)?;
    Ok(())
}

pub fn de_response<M: RpcMethod>(buf: &[u8]) -> Result<(), ResponseError<M>> {
    let mut de = ron::de::Deserializer::from_bytes(buf)?;
    let r = Result::<(), RpcError>::deserialize(&mut de)?;
    if let Err(RpcError::Method) = r {
        let m_err = M::Error::deserialize(&mut de)?;
        return Err(ResponseError::Method(m_err));
    }
    r.map_err(ResponseError::Rpc)
}
