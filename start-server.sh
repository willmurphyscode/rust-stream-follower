#!/bin/bash
set -e

CONFIG=release
export ROCKET_PORT="${PORT:-3000}"

rustup target add wasm32-unknown-unknown

if [ -z "$(cargo install --list | grep wasm-bindgen-cli)" ]
then
	cargo install wasm-bindgen-cli
fi

pushd "web/stream-plotter" > /dev/null
	mkdir -p www/pkg
	if [ "${CONFIG}" = "release" ]
	then
		cargo build --target=wasm32-unknown-unknown --release
		wasm-bindgen --out-dir "www/pkg" --target web target/wasm32-unknown-unknown/release/stream_plotter.wasm
	else
		cargo build --target=wasm32-unknown-unknown
		wasm-bindgen --out-dir "www/pkg" --target web ./target/wasm32-unknown-unknown/debug/stream_plotter.wasm
	fi
popd

ROCKET_PORT="$ROCKET_PORT" cargo run
