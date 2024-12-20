#!/bin/sh
set -e
set -x

TOP_DIR="$(dirname "$0")"
TOP_DIR="$(realpath "$TOP_DIR")"

cd "$TOP_DIR"
cargo build --features web --target wasm32-unknown-unknown

OUT_DIR="$TOP_DIR/target/server-dev/public"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

npm install
wasm-bindgen --target bundler --out-dir pkg --omit-default-module-path "$TOP_DIR/target/wasm32-unknown-unknown/debug/dioxus-fs.wasm"
node_modules/.bin/webpack

cp -r "$TOP_DIR/dist/"* "$OUT_DIR"
cp "$TOP_DIR/assets/header.svg" "$OUT_DIR/header-73ca13e70f7867c1.svg":
