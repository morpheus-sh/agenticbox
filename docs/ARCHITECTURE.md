# Architecture

## Components

- **Daemon** (`apps/daemon`): Rust axum server, orchestrates sandboxes and models.
- **Desktop** (`apps/desktop`): Tauri v2 + React UI.
- **Agent Runtime** (`apps/agent-runtime`): Python FastAPI server with Playwright tools.

## Crates

- `sandbox-core`: Docker integration via bollard.
- `session-manager`: SQLite-backed session storage.
- `model-router`: Unified model provider trait + OpenAI adapter.
- `tool-protocol`: Tool trait and definitions.
- `policy-engine`: Permission evaluation.
- `network-control`: Network policy enforcement.
- `fs-guard`: Filesystem path validation.

## Data Flow

Desktop UI -> Tauri -> Daemon REST/WebSocket -> Docker Sandbox -> Agent Runtime
