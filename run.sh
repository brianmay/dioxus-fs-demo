#!/usr/bin/env bash
export DIOXUS_CLI_ENABLED=true
exec cargo run --profile server-dev --verbose --features server --bin dioxus-fs-demo
