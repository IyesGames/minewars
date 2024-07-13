use mw_common::prelude::*;

pub enum ProtoState {
    AwaitingHandshake(AwaitingHandshake),
    HandshakeResponding(HandshakeResponding),
    HandshakeComplete(HandshakeComplete),
}

impl From<AwaitingHandshake> for ProtoState {
    fn from(x: AwaitingHandshake) -> ProtoState {
        ProtoState::AwaitingHandshake(x)
    }
}

impl From<HandshakeResponding> for ProtoState {
    fn from(x: HandshakeResponding) -> ProtoState {
        ProtoState::HandshakeResponding(x)
    }
}

impl From<HandshakeComplete> for ProtoState {
    fn from(x: HandshakeComplete) -> ProtoState {
        ProtoState::HandshakeComplete(x)
    }
}

impl ProtoState {
    pub fn new() -> Self {
        ProtoState::AwaitingHandshake(AwaitingHandshake)
    }
}

pub struct AwaitingHandshake;
pub struct HandshakeResponding {
    tx: quinn::SendStream,
}
pub struct HandshakeComplete;

impl AwaitingHandshake {
    pub async fn await_handshake(
        &mut self,
        conn: &quinn::Connection,
        buf_rx: &mut Vec<u8>,
    ) -> AnyResult<(HandshakeResponding, crate::ConnectHandshake)>
    {
        let (tx, mut rx) = conn.accept_bi().await?;

        let Some(len) = rx.read(buf_rx).await? else {
            bail!("Handshake request empty!");
        };

        if !buf_rx.starts_with(crate::HANDSHAKE_MAGIC) {
            bail!("Wrong handshake magic!");
        }

        let magic_len = crate::HANDSHAKE_MAGIC.len();

        let handshake: crate::ConnectHandshake =
            rmp_serde::decode::from_slice(&buf_rx[magic_len..len])?;

        dbg!(&handshake);

        Ok((
            HandshakeResponding { tx, },
            handshake,
        ))
    }
}

impl HandshakeResponding {
    pub async fn respond_handshake(
        &mut self,
        buf_tx: &mut Vec<u8>,
        response: &Result<crate::HandshakeSuccess, crate::HandshakeError>,
    ) -> AnyResult<HandshakeComplete> {
        buf_tx.clear();
        rmp_serde::encode::write(buf_tx, response)?;
        self.tx.write_all(buf_tx).await?;
        self.tx.finish()?;
        Ok(HandshakeComplete)
    }
}
