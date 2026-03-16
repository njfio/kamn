# Live Task-Lifecycle MCP Parity Slice

This runbook documents one bounded MCP-agent parity slice for live task lifecycle on current `main`. It is anchored to the checked-in MCP-agent S-04 probe in `kamn-e2e-harness`. It proves one local-heavy MCP execution lane against a running local Kolme runtime and local KAMN API runtime. It does not prove crash recovery, Solana-backed settlement, bridge settlement, CLI parity, or production readiness.

## Scope
- one real local-heavy MCP-agent S-04 probe on current `main`
- local Kolme plus local KAMN API runtime on loopback
- explicit parity proof against the existing live task-lifecycle slice

## Proof Anchors
- `crates/kamn-e2e-harness/src/drivers/mcp_agent/live_probe_tranche_one/task_lifecycle_probe.rs`
- `crates/kamn-e2e-harness/tests/live_s04_mcp_agent_execution.rs`
- `docs/validation/live-task-lifecycle-slice.md`

## What This Proves
- the checked-in MCP-agent driver can execute S-04 task lifecycle on current `main`
- the proof is exercised through the explicit ignored integration test `integration_live_s04_mcp_agent_task_lifecycle_probe_against_local_runtime`
- current `main` has one operator-comprehensible MCP parity lane for the same bounded live task-lifecycle slice already proven through `sdk-direct`

## Parity Bound
- this slice adds MCP-agent parity only
- it does not widen the underlying S-04 claim beyond the existing bounded local-heavy task-lifecycle lane

## What This Does Not Prove
- does not prove crash recovery
- does not prove Solana-backed settlement
- does not prove bridge settlement
- does not prove CLI parity
- does not prove production readiness
- does not prove external-chain settlement

## Local Setup
Build the required binaries:

```bash
cargo build -p kamn-node -p kamn-mcp-server -p kamn-e2e-harness
```

Build local Kolme once:

```bash
git clone https://github.com/fpco/kolme /tmp/kolme
cd /tmp/kolme
RUSTFLAGS="-C link-arg=-fuse-ld=bfd" cargo build --release -p example-p2p
```

Start Kolme API:

```bash
/tmp/kolme/target/release/example-p2p api-server --bind 127.0.0.1:13100
```

Start one local KAMN API node:

```bash
KAMN_SERVICE_API_TLS_MODE=disabled target/debug/kamn-node \
  --runtime-mode api \
  --role processor \
  --api-bind 127.0.0.1:18180 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-mcp-s04
```

## Required Environment
Export the live MCP parity env before running the proof:

```bash
export KAMN_E2E_MCP_AGENT_LIVE=true
export KAMN_E2E_MCP_AGENT_BINARY=/home/n/Code/kamn/target/debug/kamn-mcp-server
export KAMN_ENDPOINT=http://127.0.0.1:18180
export KAMN_KOLME_ENDPOINT=http://127.0.0.1:13100
export KAMN_AGENT_NAME=kamn-live-mcp-s04-proof
export KAMN_AGENT_KEY_FILE=/tmp/kamn-live-mcp-s04.key
```

## Proof Command
Run the explicit ignored proof test:

```bash
cargo test -p kamn-e2e-harness \
  --test live_s04_mcp_agent_execution \
  integration_live_s04_mcp_agent_task_lifecycle_probe_against_local_runtime \
  -- --ignored --exact --nocapture
```

## Expected Evidence
- the ignored proof test passes for `S-04`
- the live MCP driver invokes `probe-create-task`, `probe-fund-escrow`, `probe-accept-task`, `probe-complete-task`, and `probe-release-escrow`
- MCP output includes non-empty `task_id` and `escrow_id`
- MCP output returns the `state` values required by the checked-in probe validators
- the test materializes deterministic key material at `KAMN_AGENT_KEY_FILE` from `KAMN_AGENT_NAME` before launching `kamn-mcp-server`

## Notes
- live task-lifecycle MCP parity slice: `docs/validation/live-task-lifecycle-mcp-parity-slice.md`
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This slice is parity-only for the bounded live task-lifecycle lane already proven through `sdk-direct`.
