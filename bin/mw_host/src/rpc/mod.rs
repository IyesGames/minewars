use crate::prelude::*;

use mw_common::net::*;
use mw_proto_hostrpc::{RpcMethodName, RpcError};
use rustls::ServerConfig;

pub async fn rpc_main(
    mut config: Arc<Config>,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    mut reload_rx: tokio::sync::broadcast::Receiver<Arc<Config>>
) {
    info!("RPC Server initializing...");

    let mut jhs_listeners = vec![];

    loop {
        let (listener_kill_tx, _) = tokio::sync::broadcast::channel(1);

        if config.rpc.enable {
            match load_server_crypto(
                &config.rpc.cert,
                &config.rpc.key,
                config.rpc.require_client_cert,
                &config.rpc.client_ca,
            ).await {
                Ok(crypto) => {
                    info!("RPC crypto (certs and keys) loaded.");
                    for addr in config.rpc.listen.iter() {
                        let jh = tokio::spawn(rpc_listener(config.clone(), listener_kill_tx.subscribe(), crypto.clone(), *addr));
                        jhs_listeners.push(jh);
                    }
                }
                Err(e) => {
                    error!("RPC crypto (certs/keys) failed to load: {}", e);
                }
            }
        } else {
            info!("RPC disabled in config.");
        }

        tokio::select! {
            Ok(()) = shutdown_rx.recv() => {
                listener_kill_tx.send(()).ok();
                for jh in jhs_listeners.drain(..) {
                    jh.await.ok();
                }
                break;
            }
            Ok(newconfig) = reload_rx.recv() => {
                config = newconfig;
                // stop all existing listeners and create new ones next loop
                listener_kill_tx.send(()).ok();
                // wait for the old ones to stop
                for jh in jhs_listeners.drain(..) {
                    jh.await.ok();
                }
            }
        }
    }
}

async fn rpc_listener(
    config: Arc<Config>,
    mut kill_rx: tokio::sync::broadcast::Receiver<()>,
    crypto: Arc<ServerConfig>,
    addr: SocketAddr,
) {
    let endpoint = match setup_quic_server(crypto, addr) {
        Ok(endpoint) => endpoint,
        Err(e) => {
            error!("Failed to create QUIC Endpoint: {}", e);
            return;
        }
    };

    info!("Listening for incoming RPC connections on: {}", addr);

    loop {
        tokio::select! {
            Ok(()) = kill_rx.recv() => {
                break;
            }
            connecting = endpoint.accept() => {
                match connecting {
                    Some(connecting) => {
                        let config = config.clone();
                        tokio::spawn(async {
                            if let Err(e) = rpc_handle_connection(config, connecting).await {
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
    connecting: quinn::Connecting,
) -> AnyResult<()> {
    let addr_remote = connecting.remote_address();
    if !check_list(config.rpc.ip_control, config.rpc.ip_list.temporary_todo_unwrap(), &addr_remote.ip()) {
        info!("Ignoring incoming RPC connection from banned IP: {}", addr_remote);
        return Ok(());
    }
    info!("Incoming RPC connection from: {}", addr_remote);
    let conn = connecting.await?;
    info!("RPC connected: {}", addr_remote);

    loop {
        match conn.accept_bi().await {
            Ok((mut tx, rx)) => {
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
