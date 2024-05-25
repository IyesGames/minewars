use mw_common::prelude::*;

pub enum ProtoState {
    Start(ProtoStart),
    HandshakeSent(AwaitingHandshake),
    HandshakeComplete(HandshakeComplete),
}

impl From<ProtoStart> for ProtoState {
    fn from(x: ProtoStart) -> ProtoState {
        ProtoState::Start(x)
    }
}

impl From<AwaitingHandshake> for ProtoState {
    fn from(x: AwaitingHandshake) -> ProtoState {
        ProtoState::HandshakeSent(x)
    }
}

impl From<HandshakeComplete> for ProtoState {
    fn from(x: HandshakeComplete) -> ProtoState {
        ProtoState::HandshakeComplete(x)
    }
}

impl ProtoState {
    pub fn new() -> Self {
        ProtoState::Start(ProtoStart)
    }
}

pub struct ProtoStart;

pub struct AwaitingHandshake {
    rx: quinn::RecvStream,
}

pub struct HandshakeComplete;

impl ProtoStart {
    pub async fn send_handshake(
        &mut self,
        conn: &quinn::Connection,
        buf_tx: &mut Vec<u8>,
        handshake: super::ConnectHandshake,
    ) -> AnyResult<AwaitingHandshake>
    {
        let (mut tx, rx) = conn.open_bi().await?;
        buf_tx.clear();
        buf_tx.extend_from_slice(crate::HANDSHAKE_MAGIC);
        rmp_serde::encode::write(buf_tx, &handshake)?;
        tx.write_all(buf_tx).await?;
        Ok(AwaitingHandshake { rx })
    }
}

impl AwaitingHandshake {
    pub async fn await_handshake(
        &mut self,
        buf_rx: &mut Vec<u8>,
    ) -> AnyResult<HandshakeComplete>
    {
        let Some(len) = self.rx.read(buf_rx).await? else {
            bail!("Handshake response empty!");
        };
        let handshake_result: Result<crate::HandshakeSuccess, crate::HandshakeError> =
            rmp_serde::decode::from_slice(&buf_rx[..len])?;

        dbg!(handshake_result);

        Ok(HandshakeComplete)
    }
}

