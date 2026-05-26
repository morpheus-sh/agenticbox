#!/usr/bin/env bash
set -e

echo "Setting up Local AI Agent Runtime..."

echo "Installing Rust crates..."
cargo fetch

echo "Installing frontend dependencies..."
cd apps/desktop
pnpm install
cd ../..

echo "Setting up Python runtime..."
cd apps/agent-runtime
python -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
playwright install chromium
cd ../..

mkdir -p data

echo "Setup complete."
