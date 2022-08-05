# Makefile to help call `cargo` as appropriate for different builds
# Assumed to be run on Linux
default: r

TARGET_DIR_WIN := /btrfs/cargo-target/x86_64-pc-windows-msvc
CARGO_OPTS_WIN := --target x86_64-pc-windows-msvc
OUTPUT_BIN_WIN := minewars.exe

TARGET_DIR_LIN := /btrfs/cargo-target/
CARGO_OPTS_LIN := 
OUTPUT_BIN_LIN := minewars

# check (proprietary)
c:
	cargo check ${CARGO_OPTS_LIN} --features proprietary,dev,iyesengine/dynamic

# check (free)
cf:
	cargo check ${CARGO_OPTS_LIN} --features dev,iyesengine/dynamic

# dev build (native (Linux)) (proprietary)
dbg:
	cargo build ${CARGO_OPTS_LIN} --features proprietary,dev,iyesengine/dynamic

# dev build (native (Linux)) (free)
dbgf:
	cargo build ${CARGO_OPTS_LIN} --features dev,iyesengine/dynamic

# run dev-build (native (Linux)) (proprietary)
r:
	cargo run ${CARGO_OPTS_LIN} --features proprietary,dev,iyesengine/dynamic

# run dev-build (native (Linux)) (free)
rf:
	cargo run ${CARGO_OPTS_LIN} --features dev,iyesengine/dynamic

# release build (native (Linux)) (proprietary)
rel:
	cargo build ${CARGO_OPTS_LIN} --release --features proprietary,release
	cp ${TARGET_DIR_LIN}/release/${OUTPUT_BIN_LIN} .

# release build (native (Linux)) (free)
relf:
	cargo build ${CARGO_OPTS_LIN} --release --features release
	cp ${TARGET_DIR_LIN}/release/${OUTPUT_BIN_LIN} .

# release build (cross-compile (Windows)) (proprietary)
winrel:
	cargo build ${CARGO_OPTS_WIN} --release --features proprietary,release
	cp ${TARGET_DIR_WIN}/release/${OUTPUT_BIN_WIN} .

# release build (cross-compile (Windows)) (free)
winrelf:
	cargo build ${CARGO_OPTS_WIN} --release --features release
	cp ${TARGET_DIR_WIN}/release/${OUTPUT_BIN_WIN} .

# Build and Run Windows EXEs (dev builds)!:
# Assumes we can run Windows EXEs from Linux
# (only works if we are inside WSL or have Wine available)

# run dev-build (cross-compile (Windows)) (proprietary)
winr:
	cargo build ${CARGO_OPTS_WIN} --features proprietary,dev
	cp ${TARGET_DIR_WIN}/debug/${OUTPUT_BIN_WIN} .
	./${OUTPUT_BIN_WIN}

# run dev-build (cross-compile (Windows)) (proprietary)
winrf:
	cargo build ${CARGO_OPTS_WIN} --features dev
	cp ${TARGET_DIR_WIN}/debug/${OUTPUT_BIN_WIN} .
	./${OUTPUT_BIN_WIN}
