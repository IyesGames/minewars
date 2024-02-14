use mw_host::prelude::*;

use clap::Parser;

fn main() {
    let args = mw_host::cli::Args::parse();

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
             use std::sync::atomic::*;
             static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
             let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
             format!("minewars-host-worker-{}", id)
        })
        .build()
        .expect("Cannot create tokio runtime!");

    rt.block_on(async_main(args));
}

async fn async_main(args: mw_host::cli::Args) {
    let rt = ManagedRuntime::new();

    let persist = mw_host::init(rt.clone()).await;
    #[cfg(feature = "proprietary")]
    let persist_proprietary = mw_host_proprietary::init(rt.clone()).await;

    loop {
        persist.do_soft_reset();
        #[cfg(feature = "proprietary")]
        persist_proprietary.do_soft_reset();

        let softreset = rt.child_token();

        let config = match mw_host::load_config(&args.config, &args).await {
            Ok(config) => config,
            Err(e) => {
                error!("Error loading config file: {:#}", e);
                if rt.has_tasks() {
                    let sec_retry = 15;
                    info!("Server has active tasks. Will not shut down; retrying config load in {} seconds.", sec_retry);
                    tokio::time::sleep(Duration::from_secs(sec_retry)).await;
                    continue;
                } else {
                    break;
                }
            }
        };

        #[allow(unused_variables)]
        let for_proprietary = mw_host::setup_with_config(
            rt.clone(), &persist, softreset.clone(), config.clone()
        ).await;
        #[cfg(feature = "proprietary")]
        mw_host_proprietary::setup_with_config(
            rt.clone(), &persist_proprietary, softreset.clone(), config.clone(), for_proprietary
        ).await;

        tokio::select! {
            _ = softreset.cancelled() => {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            _ = rt.listen_shutdown() => {
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Ctrl-C interrupt received!");
                break;
            }
        }
    }

    info!("Shutting down...");
    rt.trigger_shutdown();
    rt.wait_shutdown().await;
    info!("Shutdown complete.");
}
