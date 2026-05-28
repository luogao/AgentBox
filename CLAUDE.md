# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
cargo build --release              # Build all workspace crates
cargo test                         # Run all tests
cargo test -p control-plane        # Run control-plane tests only
cargo test -p sidecar              # Run sidecar tests only
cargo run -p control-plane          # Start control plane locally
```

Database tests use `sqlite::memory:`, so they work without any setup.

## Docker

```bash
# Build the agent container image (sidecar + entrypoint)
docker build -t agent-sandbox:latest -f agent-image/Dockerfile .

# Build the control-plane image
docker build -t agentbox-control:latest -f Dockerfile .

# Run full stack (control plane + agent container)
docker compose up -d

# Run with agent profile
docker compose --profile agent up -d
```

## Architecture

This is a Rust workspace (`agentbox`) with three components:

### Control Plane (`control-plane/`)
The main REST API service (Axum 0.8) that manages Docker container lifecycles. Connects to Docker via the local socket using Bollard. Stores container metadata in SQLite (sqlx 0.8).

- **Routes** (defined in `control-plane/src/main.rs`):
  - `GET /health` — health check
  - `POST /api/containers` — create and start a new agent container
  - `GET /api/containers/{id}` — get container metadata
  - `DELETE /api/containers/{id}` — stop + remove container + delete DB record
  - `POST /api/containers/{id}/status` — receive status reports from sidecar
- **LifecycleManager** runs as a background task (30s tick), checking idle timeout and max lifetime for all active containers. When exceeded, it stops/removes the Docker container and marks it `Stopped` in the DB.
- **Auth middleware** (`control-plane/src/auth.rs`) checks the `Authorization: Bearer <key>` header on all routes except `/health`. If `API_KEY` env var is not set, auth is skipped entirely (development mode).

### Sidecar (`sidecar/`)
Runs inside each agent container. Reports progress and heartbeat back to the control plane via HTTP POST to `/api/containers/{id}/status`. Currently simulates a task in 5 steps; in production this would invoke the Claude Agent SDK.

### Agent Image (`agent-image/`)
Docker image for agent containers. Contains the sidecar binary. The entrypoint script (`entrypoint.sh`) starts the sidecar in background, then clones skill repos from `$SKILL_REPOS` (comma-separated Git URLs) into `/workspace/skills/`.

## Key Data Flow

1. User POSTs to `/api/containers` with task + skill repos + resource limits
2. Control plane generates a UUID, creates a Docker container named `agent-{uuid}`, injects env vars (TASK, CONTAINER_ID, CONTROL_PLANE_URL, SKILL_REPOS)
3. Sidecar starts inside the container, reports status back to control plane
4. LifecycleManager periodically sweeps active containers, stopping idle/expired ones

## Environment Variables

**Control Plane**: `DATABASE_URL` (default `sqlite:agent_sandbox.db?mode=rwc`), `SERVER_ADDR` (default `0.0.0.0:8080`), `AGENT_IMAGE` (default `agent-sandbox:latest`), `API_KEY` (optional; if set, all non-/health routes require `Authorization: Bearer <key>`), `RUST_LOG` (default `info`)

**Sidecar/Container**: `CONTAINER_ID` (required), `TASK`, `CONTROL_PLANE_URL`, `SKILL_REPOS`, `ANTHROPIC_API_KEY`
