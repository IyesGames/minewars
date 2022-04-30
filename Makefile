default: run

c:
	cargo check --features proprietary,dev

debug:
	cargo build --features proprietary,dev

run:
	cargo run --features proprietary,dev,iyesengine/dynamic

release:
	cargo build --release --features proprietary,release
