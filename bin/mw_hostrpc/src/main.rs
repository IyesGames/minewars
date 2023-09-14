use std::{net::{IpAddr, SocketAddr}, path::PathBuf, sync::Arc};
use anyhow::Result as AnyResult;

use clap::Parser;
use rustls::{Certificate, RootCertStore};

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, value_name = "IP")]
    server: IpAddr,
    #[arg(short, long, value_name = "PORT")]
    port: u16,
    #[arg(long, value_name = "DER_FILE")]
    ca_server: PathBuf,
}

async fn connect_sendrpc(args: Args) -> AnyResult<()> {
    use mw_proto_hostrpc::methods::kill_session::KillSession;

    let ca_bytes = tokio::fs::read(&args.ca_server).await?;
    let ca = Certificate(ca_bytes);
    let mut roots = RootCertStore::empty();
    roots.add(&ca)?;
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let config = quinn::ClientConfig::new(Arc::new(crypto));
    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;
    endpoint.set_default_client_config(config);
    let conn = endpoint.connect(SocketAddr::new(args.server, args.port), "auth.iyes.games")?.await?;
    let (mut tx, rx) = conn.open_bi().await?;
    let mut buf = vec![];
    mw_proto_hostrpc::ser_request(&mut buf, &KillSession {
        session_id: 1503,
    })?;
    tx.write_all(&buf).await?;
    tx.finish().await?;
    let buf = rx.read_to_end(4096).await?;
    let response = mw_proto_hostrpc::de_response::<KillSession>(&buf)?;
    dbg!(response);
    Ok(())
}

fn main() {
    let args = Args::parse();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().unwrap();

    rt.block_on(connect_sendrpc(args)).unwrap();
}
