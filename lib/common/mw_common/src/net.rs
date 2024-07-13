use crate::prelude::*;

pub mod prelude {
    pub use tokio::sync::broadcast::{Sender as TxBroadcast, Receiver as RxBroadcast};
    pub use tokio::sync::mpsc::{Sender as TxMpsc, Receiver as RxMpsc, UnboundedSender as TxMpscU, UnboundedReceiver as RxMpscU};
    pub use tokio::sync::oneshot::{Sender as TxOneshot, Receiver as RxOneshot};
    pub use tokio_util::sync::CancellationToken;
    pub use super::ManagedRuntime;
}

use quinn::Endpoint;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use tokio_util::task::TaskTracker;

#[derive(Clone)]
pub struct ManagedRuntime {
    shutdown_token: CancellationToken,
    tracker: Arc<TaskTracker>,
}

impl ManagedRuntime {
    pub fn new() -> Self {
        ManagedRuntime {
            shutdown_token: CancellationToken::new(),
            tracker: Arc::new(TaskTracker::new()),
        }
    }
    pub fn token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }
    pub fn child_token(&self) -> CancellationToken {
        self.shutdown_token.child_token()
    }
    pub fn trigger_shutdown(&self) {
        self.shutdown_token.cancel();
    }
    pub async fn listen_shutdown(&self) {
        self.shutdown_token.cancelled().await;
    }
    pub async fn wait_shutdown(&self) {
        self.tracker.close();
        self.tracker.wait().await;
    }
    pub fn has_tasks(&self) -> bool {
        !self.tracker.is_empty()
    }
    pub fn spawn<F>(&self, task: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tracker.spawn(task)
    }
}

/// How to interpret a list of restrictions for security
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum ControlListMode {
    /// Everything else, except for what is in the list, is allowed
    Denylist,
    /// Only what is in the list is allowed
    Allowlist,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
pub struct ServerSettings {
    pub server_certs: Vec<PathBuf>,
    pub server_key: PathBuf,
    pub client_ca: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
pub struct ClientSettings {
    pub client_certs: Vec<PathBuf>,
    pub client_key: Option<PathBuf>,
    pub server_ca: Vec<PathBuf>,
}

pub async fn setup_quic(
    my_addr: SocketAddr,
    server_settings: Option<&ServerSettings>,
    client_settings: Option<&ClientSettings>,
) -> AnyResult<Endpoint> {
    let mut endpoint = if let Some(ss) = server_settings {
        let b = if !ss.client_ca.is_empty() {
            let mut roots = rustls::RootCertStore::empty();
            for path in ss.client_ca.iter() {
                let client_ca = tokio::fs::read(path).await
                    .context("Cannot load Client CA cert")?;
                roots.add(CertificateDer::from(client_ca))
                    .context("Cannot add Client CA cert to root store")?;
            }
            let verifier = rustls::server::WebPkiClientVerifier::builder(roots.into())
                .build()
                .context("Cannot create client cert verifier")?;
            rustls::ServerConfig::builder()
                .with_client_cert_verifier(verifier)
        } else {
            rustls::ServerConfig::builder()
                .with_no_client_auth()
        };
        let mut server_certs = Vec::with_capacity(ss.server_certs.len());
        for path in ss.server_certs.iter() {
            let server_cert = tokio::fs::read(path).await
                .context("Cannot load server cert")?;
            server_certs.push(CertificateDer::from(server_cert));
        }
        let server_key = tokio::fs::read(&ss.server_key).await
            .context("Cannot load server private key")?;
        let server_crypto = b.with_single_cert(
            server_certs,
            PrivatePkcs8KeyDer::from(server_key).into(),
        ).context("Cannot create server crypto")?;
        let server_crypto = quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)
            .context("Cannot create server crypto")?;
        let server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));
        Endpoint::server(server_config, my_addr)
    } else {
        Endpoint::client(my_addr)
    }.context("Cannot create QUIC endpoint")?;

    if let Some(cs) = client_settings {
        let mut roots = rustls::RootCertStore::empty();
        for path in cs.server_ca.iter() {
            let server_ca = tokio::fs::read(path).await
                .context("Cannot load Server CA cert")?;
            roots.add(CertificateDer::from(server_ca))
                .context("Cannot add Server CA cert to root store")?;
        }
        let b = rustls::ClientConfig::builder()
            .with_root_certificates(roots);
        let client_crypto = if let Some(path) = &cs.client_key {
            let mut client_certs = Vec::with_capacity(cs.client_certs.len());
            for path in cs.client_certs.iter() {
                let client_cert = tokio::fs::read(path).await
                    .context("Cannot load client cert")?;
                client_certs.push(CertificateDer::from(client_cert));
            }
            let client_key = tokio::fs::read(path).await
                .context("Cannot load client private key")?;
            b.with_client_auth_cert(
                client_certs,
                PrivatePkcs8KeyDer::from(client_key).into(),
            ).context("Cannot create client crypto")?
        } else {
            b.with_no_client_auth()
        };
        let client_crypto = quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)
            .context("Cannot create client crypto")?;
        let client_config = quinn::ClientConfig::new(Arc::new(client_crypto));
        endpoint.set_default_client_config(client_config);
    }
    Ok(endpoint)
}
