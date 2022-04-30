default: run

c:
	cargo check --features proprietary,dev

cf:
	cargo check --features dev

debug:
	cargo build --features proprietary,dev

debug_free:
	cargo build --features dev

run:
	cargo run --features proprietary,dev,iyesengine/dynamic

run_free:
	cargo run --features dev,iyesengine/dynamic

release:
	cargo build --release --features proprietary,release

release_free:
	cargo build --release --features release
