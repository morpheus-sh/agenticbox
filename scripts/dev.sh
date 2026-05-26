#!/usr/bin/env bash
set -e

echo "Starting local development stack..."

mkdir -p data

export DATABASE_URL="sqlite:data/sessions.db"
export RUST_LOG=info

cargo run --bin daemon &
DAEMON_PID=$!

cd apps/desktop
pnpm tauri dev &
TAURI_PID=$!

trap "kill $DAEMON_PID $TAURI_PID" EXIT

wait
