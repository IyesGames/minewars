use crate::prelude::*;

pub mod prelude {
    pub use tokio::sync::broadcast::{Sender as TxBroadcast, Receiver as RxBroadcast};
    pub use tokio::sync::mpsc::{Sender as TxMpsc, Receiver as RxMpsc, UnboundedSender as TxMpscU, UnboundedReceiver as RxMpscU};
    pub use tokio::sync::oneshot::{Sender as TxOneshot, Receiver as RxOneshot};
    pub use tokio_util::sync::CancellationToken;
    pub use super::ManagedRuntime;
}

use quinn::Endpoint;
use rustls::{Certificate, PrivateKey, RootCertStore, server::AllowAnyAuthenticatedClient};
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

pub async fn load_server_crypto(
    my_certs: &[impl AsRef<Path>],
    my_key: impl AsRef<Path>,
    client_verification: bool,
    client_ca: impl AsRef<Path>,
) -> AnyResult<Arc<rustls::ServerConfig>> {
    let mut my_certs_data = vec![];
    for cert in my_certs {
        my_certs_data.push(load_cert(cert.as_ref()).await?);
    }
    let my_key_data = load_key(my_key.as_ref()).await?;
    let client_ca_data = if client_verification {
        Some(load_cert(client_ca.as_ref()).await?)
    } else {
        None
    };
    setup_server_crypto(&my_certs_data, &my_key_data, client_ca_data.as_ref())
}

pub async fn load_client_crypto(
    server_ca: impl AsRef<Path>,
    client_verification: bool,
    my_certs: &[impl AsRef<Path>],
    my_key: impl AsRef<Path>,
) -> AnyResult<Arc<rustls::ClientConfig>> {
    let server_ca_data = load_cert(server_ca.as_ref()).await?;
    if client_verification {
        let mut my_certs_data = vec![];
        for cert in my_certs {
            my_certs_data.push(load_cert(cert.as_ref()).await?);
        }
        let my_key_data = load_key(my_key.as_ref()).await?;
        setup_client_crypto(Some((&my_certs_data, &my_key_data)), &server_ca_data)
    } else {
        setup_client_crypto(None, &server_ca_data)
    }
}

pub fn setup_server_crypto(
    my_certs: &[Certificate],
    my_key: &PrivateKey,
    client_ca: Option<&Certificate>,
) -> AnyResult<Arc<rustls::ServerConfig>> {
    let crypto = rustls::ServerConfig::builder()
        .with_safe_defaults();

    let crypto = match client_ca {
        Some(client_ca) => {
            let mut roots = RootCertStore::empty();
            roots.add(client_ca)?;
            crypto.with_client_cert_verifier(Arc::new(AllowAnyAuthenticatedClient::new(roots)))
        },
        None => {
            crypto.with_no_client_auth()
        },
    };

    let crypto = crypto.with_single_cert(
        my_certs.into_iter().cloned().collect(),
        my_key.clone()
    )?;

    Ok(Arc::new(crypto))
}

pub fn setup_client_crypto(
    my_certs_key: Option<(&[Certificate], &PrivateKey)>,
    server_ca: &Certificate,
) -> AnyResult<Arc<rustls::ClientConfig>> {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults();

    let mut roots = RootCertStore::empty();
    roots.add(server_ca)?;
    let crypto = crypto.with_root_certificates(roots);

    let crypto = match my_certs_key {
        Some((my_certs, my_key)) => {
            crypto.with_client_auth_cert(
                my_certs.into_iter().cloned().collect(),
                my_key.clone()
            )?
        },
        None => {
            crypto.with_no_client_auth()
        },
    };

    Ok(Arc::new(crypto))
}

pub fn setup_quic_server(
    crypto: Arc<rustls::ServerConfig>,
    my_addr: SocketAddr,
) -> AnyResult<Endpoint> {
    let config = quinn::ServerConfig::with_crypto(crypto);
    let endpoint = Endpoint::server(config, my_addr)?;
    Ok(endpoint)
}

pub fn setup_quic_client(
    crypto: Arc<rustls::ClientConfig>,
    my_addr: SocketAddr,
) -> AnyResult<Endpoint> {
    let config = quinn::ClientConfig::new(crypto);
    let mut endpoint = Endpoint::client(my_addr)?;
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}
