default: run

TARGET_DIR  := /var/tmp/cargo/x86_64-pc-windows-msvc
CARGO_OPTS := --target x86_64-pc-windows-msvc
OUTPUT_BIN := minewars.exe

c:
	cargo check ${CARGO_OPTS} --features proprietary,dev

cf:
	cargo check ${CARGO_OPTS} --features dev

debug:
	cargo build ${CARGO_OPTS} --features proprietary,dev
	cp ${TARGET_DIR}/debug/${OUTPUT_BIN} .

debug_free:
	cargo build ${CARGO_OPTS} --features dev
	cp ${TARGET_DIR}/debug/${OUTPUT_BIN} .

run:
	cargo build ${CARGO_OPTS} --features proprietary,dev
	cp ${TARGET_DIR}/debug/${OUTPUT_BIN} .
	./${OUTPUT_BIN}

run_free:
	cargo build ${CARGO_OPTS} --features dev
	cp ${TARGET_DIR}/debug/${OUTPUT_BIN} .
	./${OUTPUT_BIN}

release:
	cargo build ${CARGO_OPTS} --release --features proprietary,release
	cp ${TARGET_DIR}/release/${OUTPUT_BIN} .

release_free:
	cargo build ${CARGO_OPTS} --release --features release
	cp ${TARGET_DIR}/release/${OUTPUT_BIN} .
