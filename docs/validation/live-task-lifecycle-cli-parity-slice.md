# Live Task-Lifecycle CLI Parity Slice

This runbook documents one bounded CLI-scripted parity slice for live task lifecycle on current `main`. It is anchored to the checked-in CLI-scripted S-04 probe in `kamn-e2e-harness`. It proves one local-heavy CLI execution lane against a running local Kolme runtime and local KAMN API runtime. It does not prove crash recovery, Solana-backed settlement, bridge settlement, MCP parity, or production readiness.

## Scope
- one real local-heavy CLI-scripted S-04 probe on current `main`
- local Kolme plus local KAMN API runtime on loopback
- explicit parity proof against the existing live task-lifecycle slice

## Proof Anchors
- `crates/kamn-e2e-harness/src/drivers/cli_scripted/live_probe_tranche_one/channel_task_probes/task_lifecycle_probe.rs`
- `crates/kamn-e2e-harness/tests/live_s04_cli_scripted_execution.rs`
- `docs/validation/live-task-lifecycle-slice.md`

## What This Proves
- the checked-in CLI-scripted driver can execute S-04 task lifecycle on current `main`
- the proof is exercised through the explicit ignored integration test `integration_live_s04_cli_scripted_task_lifecycle_probe_against_local_runtime`
- current `main` has one operator-comprehensible CLI parity lane for the same bounded live task-lifecycle slice already proven through `sdk-direct`

## Parity Bound
- this slice adds CLI parity only
- it does not widen the underlying S-04 claim beyond the existing bounded local-heavy task-lifecycle lane

## What This Does Not Prove
- does not prove crash recovery
- does not prove Solana-backed settlement
- does not prove bridge settlement
- does not prove MCP parity
- does not prove production readiness
- does not prove external-chain settlement

## Local Setup
Build the required binaries:

```bash
cargo build -p kamn-node -p kamn-cli -p kamn-e2e-harness
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
  --storage-dir /tmp/kamn-node-live-cli-s04
```

## Required Environment
Export the live CLI parity env before running the proof:

```bash
export KAMN_E2E_CLI_SCRIPTED_LIVE=true
export KAMN_E2E_CLI_BINARY=/home/n/Code/kamn/target/debug/kamn-cli
export KAMN_ENDPOINT=http://127.0.0.1:18180
export KAMN_KOLME_ENDPOINT=http://127.0.0.1:13100
export KAMN_AGENT_NAME=kamn-live-cli-s04-proof
```

## Proof Command
Run the explicit ignored proof test:

```bash
cargo test -p kamn-e2e-harness \
  --test live_s04_cli_scripted_execution \
  integration_live_s04_cli_scripted_task_lifecycle_probe_against_local_runtime \
  -- --ignored --exact --nocapture
```

## Expected Evidence
- the ignored proof test passes for `S-04`
- the live CLI driver invokes `create-task`, `fund-escrow`, `accept-task`, `complete-task`, and `release-escrow`
- CLI output includes non-empty `task_id` and `escrow_id`
- CLI output returns the `state` values required by the checked-in probe validators

## Notes
- live task-lifecycle CLI parity slice: `docs/validation/live-task-lifecycle-cli-parity-slice.md`
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This slice is parity-only for the bounded live task-lifecycle lane already proven through `sdk-direct`.
