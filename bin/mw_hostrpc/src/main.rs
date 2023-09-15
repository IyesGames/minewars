use mw_common::prelude::*;
use mw_common::net::*;

use clap::Parser;

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

    let crypto = load_client_crypto(&args.ca_server, false, &[""], "").await?;
    let endpoint = setup_quic_client(crypto, "0.0.0.0:0".parse().unwrap())?;

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
