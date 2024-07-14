use crate::prelude::*;

use mw_common::net::*;
use mw_proto_hostrpc::{RpcMethodName, RpcError};
use quinn::crypto::rustls::QuicServerConfig;

pub async fn rpc_main(
    config: Arc<Config>,
    rt: ManagedRuntime,
    softreset: CancellationToken,
) {
    if config.rpc.enable {
        info!("RPC Server initializing...");
        let server_settings = ServerSettings {
            server_certs: config.rpc.cert.clone(),
            server_key: config.rpc.key.clone(),
            client_ca: if config.rpc.require_client_cert {
                config.rpc.client_ca.clone()
            } else {
                vec![]
            },
        };
        match load_server_crypto(
            &server_settings,
        ).await {
            Ok(crypto) => {
                info!("RPC crypto (certs and keys) loaded.");
                for addr in config.rpc.listen.iter() {
                    rt.spawn(
                        rpc_listener(
                            config.clone(),
                            rt.clone(),
                            softreset.clone(),
                            crypto.clone(),
                            *addr,
                        )
                    );
                }
            }
            Err(e) => {
                error!("RPC crypto (certs/keys) failed to load: {}", e);
            }
        }
    } else {
        info!("RPC disabled in config.");
    }
}

async fn rpc_listener(
    config: Arc<Config>,
    rt: ManagedRuntime,
    softreset: CancellationToken,
    crypto: Arc<QuicServerConfig>,
    addr: SocketAddr,
) {
    let endpoint = match setup_quic(addr, Some(crypto), None) {
        Ok(endpoint) => endpoint,
        Err(e) => {
            error!("Failed to create QUIC Endpoint: {}", e);
            return;
        }
    };

    info!("Listening for incoming RPC connections on: {}", addr);

    loop {
        tokio::select! {
            _ = softreset.cancelled() => {
                break;
            }
            _ = rt.listen_shutdown() => {
                break;
            }
            incoming = endpoint.accept() => {
                match incoming {
                    Some(incoming) => {
                        let config = config.clone();
                        tokio::spawn(async {
                            if let Err(e) = rpc_handle_connection(config, incoming).await {
                                error!("RPC connection error: {}", e);
                            }
                        });
                    }
                    None => {
                        error!("RPC endpoint for {} closed!", addr);
                        break;
                    }
                }
            }
        }
    }
}

async fn rpc_handle_connection(
    config: Arc<Config>,
    incoming: quinn::Incoming,
) -> AnyResult<()> {
    let addr_remote = incoming.remote_address();
    if !check_list(config.rpc.ip_control, config.rpc.ip_list.temporary_todo_unwrap(), &addr_remote.ip()) {
        info!("Ignoring incoming RPC connection from banned IP: {}", addr_remote);
        incoming.ignore();
        return Ok(());
    }
    info!("Incoming RPC connection from: {}", addr_remote);
    let conn = incoming.await?;
    info!("RPC connected: {}", addr_remote);

    loop {
        match conn.accept_bi().await {
            Ok((mut tx, mut rx)) => {
                const IN_MAX_LEN: usize = 64 * 1024;
                match rx.read_to_end(IN_MAX_LEN).await {
                    Ok(mut buf) => {
                        match rpc_handle_request(&config, &mut buf) {
                            Ok(()) => {
                                if let Err(e) = tx.write_all(&buf).await {
                                    warn!("RPC Write error: {}", e);
                                    // should we break here? i think no,
                                    // client might want to disregard our write,
                                    // but open more streams for more requests
                                }
                            }
                            Err(e) => {
                                warn!("Received erroneous request over RPC. Terminating client for security. Error: {}", e);
                                conn.close(1u8.into(), &[]);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("RPC Read error: {}", e);
                        // break here for security
                        // if the read errors, something is wrong with the client
                        break;
                    }
                }
            }
            Err(e) => {
                info!("RPC connection terminated: {}", e);
                break;
            }
        }
    }
    Ok(())
}

fn rpc_handle_request(
    config: &Config,
    buf: &mut Vec<u8>,
) -> AnyResult<()> {
    let mut de = ron::Deserializer::from_bytes(buf.as_mut_slice())?;
    let methodname = RpcMethodName::deserialize(&mut de)?;
    if !check_list(config.rpc.rpc_method_control, &config.rpc.rpc_methods_list, &methodname) {
        buf.clear();
        ron::ser::to_writer(buf, &RpcError::Forbidden)?;
        return Ok(());
    }
    match methodname {
        RpcMethodName::ReloadConfig => {
            let request = mw_proto_hostrpc::methods::reload_config::ReloadConfig::deserialize(&mut de)?;
            dbg!(request);
        }
        RpcMethodName::CreateSession => {
            let request = mw_proto_hostrpc::methods::create_session::CreateSession::deserialize(&mut de)?;
            dbg!(request);
        }
        RpcMethodName::KillSession => {
            let request = mw_proto_hostrpc::methods::kill_session::KillSession::deserialize(&mut de)?;
            dbg!(request);
        }
        RpcMethodName::ExpectPlayer => {
            let request = mw_proto_hostrpc::methods::expect_player::ExpectPlayer::deserialize(&mut de)?;
            dbg!(request);
        }
    }
    buf.clear();
    let r: Result<(), _> = Err(RpcError::Unsupported);
    ron::ser::to_writer(buf, &r)?;
    Ok(())
}
