default: run

c:
	cargo check --features proprietary,iyesengine/dev,iyesengine/dynamic

debug:
	cargo build --features proprietary,iyesengine/dev,iyesengine/dynamic

run:
	cargo run --features proprietary,iyesengine/dev,iyesengine/dynamic

release:
	cargo build --release --features proprietary,iyesengine/release
