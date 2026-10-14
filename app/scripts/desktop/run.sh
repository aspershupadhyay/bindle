#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
#
# Runs the desktop checks against the real app on Linux. Needs a debug build
# with the views embedded, the relay, tauri-driver and WebKitWebDriver:
#
#   (cd app && npx vite build)
#   cargo build -p bindle-app --features tauri/custom-protocol
#   cargo build -p bindle-relay
#   cargo install tauri-driver --locked
#   sudo apt-get install webkit2gtk-driver xvfb
#   xvfb-run -a app/scripts/desktop/run.sh
#
# Screenshots go to $BINDLE_SHOTS (a temporary folder when unset).
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../../.." && pwd)
work=$(mktemp -d)
export BINDLE_APP="${BINDLE_APP:-$root/target/debug/bindle-app}"
export BINDLE_SHOTS="${BINDLE_SHOTS:-$work/shots}"
export BINDLE_RELAY_URL="http://127.0.0.1:8787"
mkdir -p "$BINDLE_SHOTS" "$work/a" "$work/b"

PORT=8787 "$root/target/debug/bindle-relay" "$work/relay.db" > "$work/relay.log" 2>&1 &
relay=$!
# Each copy of Bindle keeps its data in its own home folder.
HOME="$work/a" XDG_DATA_HOME="$work/a/data" tauri-driver --port 4444 --native-port 4446 > "$work/driver-a.log" 2>&1 &
driver_a=$!
HOME="$work/b" XDG_DATA_HOME="$work/b/data" tauri-driver --port 4445 --native-port 4447 > "$work/driver-b.log" 2>&1 &
driver_b=$!
trap 'kill $relay $driver_a $driver_b 2>/dev/null || true' EXIT
sleep 2

node "$here/checks.mjs"
