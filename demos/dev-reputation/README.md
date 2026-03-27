# IPC Developer Reputation Demo

This demo implements a verifiable developer reputation pipeline on IPC: an off-chain Node.js agent collects and analyses GitHub PR activity, applies anti-gaming controls, writes a signed evidence package to Basin, and submits a compact score record to an IPC reputation actor.

## Directory layout

- `agent/` - off-chain scorer, CLI, and HTTP API
- `actor/` - Rust/WASM reputation actor
- `frontend/` - static dashboard (no build step)
- `scripts/` - deploy, scoring helper, smoke test

## Agent setup

```bash
cd demos/dev-reputation/agent
cp .env.example .env
npm install
npm start
```

### Agent environment variables

- `ANTHROPIC_API_KEY` (required)
- `AGENT_PRIVATE_KEY` (required, secp256k1 hex without `0x`)
- `GITHUB_TOKEN` (optional but recommended)
- `BASIN_API_URL` (required)
- `BASIN_BUCKET` (required)
- `IPC_RPC_URL` (required)
- `REPUTATION_ACTOR_ADDRESS` (required for on-chain submit)
- `ADMIN_ADDRESS` (required)
- `PORT` (optional, defaults to `3001`)

## Actor build

```bash
cd demos/dev-reputation/actor
cargo build --target wasm32-unknown-unknown --release
```

## Deploy actor

```bash
cd demos/dev-reputation
export ADMIN_ADDRESS=0x...
export AGENT_ADDRESS=0x...
./scripts/deploy.sh
```

`deploy.sh` builds the WASM, deploys the actor via `ipc-cli`, initializes admin + initial agent, writes `.env.actor`, and updates `frontend/config.js`.

## Score a developer

```bash
cd demos/dev-reputation
./scripts/score.sh torvalds 0x000000000000000000000000000000000000dEaD
```

## API

- `POST /score` with `{ github_handle, wallet_address }` returns `job_id`
- `GET /job/:job_id` returns status and progress
- `GET /score/:github_handle` returns latest score
- `GET /health` returns agent metadata

## Dashboard

Open `frontend/index.html` directly in a browser. The dashboard supports leaderboard + profile views and client-side verification checks (content hash, signer recovery, and block timestamp check).

## Anti-gaming model

The anti-gaming system combines LLM judgment with deterministic heuristics: relocation-without-change downweights synthetic large moves, formatter-only commits and low-signal message inflation are excluded from weighted commit counts, and generated artifacts are discounted to 0.1x line contribution so score signal stays anchored to substantive engineering work rather than surface churn.

## Tests

```bash
cd demos/dev-reputation/agent && npm test
cd ../actor && cargo test
cd .. && ./scripts/smoke_test.sh
```
