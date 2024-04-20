#!/bin/sh

export RUST_LOG="mw_host=trace,mw_hostsrv=trace"
cargo b -p mw_hostrpc &&
cp ./target/debug/mw_hostrpc . &&
exec cargo r -p mw_hostsrv --features proprietary -- "$@"
