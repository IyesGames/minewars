#!/bin/sh
cargo build \
  --config ./.cargo/config-cross.toml \
  --target x86_64-pc-windows-gnu \
  --features dev,proprietary &&
cp ./target-cross/x86_64-pc-windows-gnu/debug/minewars.exe . &&
exec ./minewars.exe "$@"
