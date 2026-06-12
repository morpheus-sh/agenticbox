#!/usr/bin/env bash
# Cloudflare Pages build script with Rust installation

set -e

echo "🔧 Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

echo "📦 Installing frontend dependencies..."
pnpm install --frozen-lockfile

echo "🦀 Building Rust daemon (release)..."
cargo build --release

echo "🎨 Building Tauri desktop..."
pnpm --filter desktop build

echo "✅ Build complete!"