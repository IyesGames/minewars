#!/bin/sh
cargo build \
  --config ./.cargo/config-cross.toml \
  --target x86_64-pc-windows-gnu \
  --features dev &&
cp ./target-cross/x86_64-pc-windows-gnu/debug/minewars_foss.exe . &&
exec ./minewars_foss.exe "$@"
