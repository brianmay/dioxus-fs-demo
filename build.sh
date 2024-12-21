#!/usr/bin/env bash
set -e
set -x

copy_hashed() {
  local filename
  local hash
  local dst
  local extension
  local name
  filename="$(basename "$1")"
  hash="$(b3sum --raw "$1" | head --bytes 6 | base64)"
  extension="${filename##*.}"
  name="${filename%.*}"
  dst="$OUT_DIR/$name-$hash.$extension"
  cp "$1" "$dst"
}

TOP_DIR="$(dirname "$0")"
TOP_DIR="$(realpath "$TOP_DIR")"

cd "$TOP_DIR"
cargo build --features web --target wasm32-unknown-unknown

OUT_DIR="$TOP_DIR/target/server-dev/public"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

#npm install
wasm-bindgen --target bundler --out-dir pkg --omit-default-module-path "$TOP_DIR/target/wasm32-unknown-unknown/debug/dioxus-fs-demo.wasm"
node_modules/.bin/webpack

cp -r "$TOP_DIR/dist/"* "$OUT_DIR"
copy_hashed "$TOP_DIR/assets/header.svg"
copy_hashed "$TOP_DIR/assets/main.css"
copy_hashed "$TOP_DIR/assets/favicon.ico"
