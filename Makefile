test:
	cargo test

build:
	cargo build --release

install: build
	cargo install --path .
