build:
	cargo build --release

wc-seq: build
	cargo run --release --package sequential -- -a app_wc -i inputs/*
