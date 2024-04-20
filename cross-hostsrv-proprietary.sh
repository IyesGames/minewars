#!/bin/sh
cargo build -p mw_hostsrv -p mw_hostrpc \
  --config ./.cargo/config-cross.toml \
  --target x86_64-pc-windows-gnu \
  --features proprietary &&
cp ./target-cross/x86_64-pc-windows-gnu/debug/mw_hostsrv.exe ./target-cross/x86_64-pc-windows-gnu/debug/mw_hostrpc.exe . &&
exec ./mw_hostsrv.exe "$@"
