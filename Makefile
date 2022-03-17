default: run

c:
	cargo check --features iyesengine/dev

debug:
	cargo build --features iyesengine/dev

run:
	cargo run --features iyesengine/dev,iyesengine/dynamic

release:
	cargo build --release --features iyesengine/release
