# Live Task Lifecycle Slice

This runbook documents one bounded live task-lifecycle slice on current `main`. It is anchored to the checked-in `sdk-direct` S-04 probe in `kamn-e2e-harness`. It proves one local-heavy `sdk-direct` S-04 execution lane against a running local Kolme runtime and local KAMN API runtime. It does not prove crash recovery, Solana-backed settlement, bridge settlement, or production readiness.

## Scope
- one real local-heavy `sdk-direct` S-04 probe on current `main`
- local Kolme plus local KAMN API runtime on loopback
- one bounded task-lifecycle lane: create task, fund escrow, accept task, complete task, release escrow

## Proof Anchors
- `crates/kamn-e2e-harness/src/scenarios/s04_task.rs`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_one/channel_task_probes.rs`
- `crates/kamn-e2e-harness/tests/live_s04_sdk_direct_execution.rs`
- `docs/validation/working-vertical-slice.md`

## What This Proves
- the checked-in `sdk-direct` driver can execute S-04 task lifecycle on current `main`
- the proof is exercised through the explicit ignored integration test `integration_live_s04_sdk_direct_task_lifecycle_probe_against_local_runtime`
- the live lane creates a real task, funds one real escrow, advances the task through accept and complete, and releases the escrow
- current `main` has one operator-comprehensible live task-lifecycle slice beyond the earlier service-api vertical slice

## What This Does Not Prove
- does not prove crash recovery
- does not prove Solana-backed settlement
- does not prove bridge settlement
- does not prove production readiness
- does not prove external-chain settlement
- does not prove MCP or CLI parity

## Local Setup
Build the required binaries:

```bash
cargo build -p kamn-node -p kamn-e2e-harness
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
  --storage-dir /tmp/kamn-node-live-s04
```

## Required Environment
Export the live `sdk-direct` env before running the proof:

```bash
export KAMN_E2E_SDK_DIRECT_LIVE=true
export KAMN_ENDPOINT=http://127.0.0.1:18180
export KAMN_KOLME_ENDPOINT=http://127.0.0.1:13100
export KAMN_AGENT_NAME=kamn-live-s04-proof
```

## Proof Command
Run the explicit ignored proof test:

```bash
cargo test -p kamn-e2e-harness \
  --test live_s04_sdk_direct_execution \
  integration_live_s04_sdk_direct_task_lifecycle_probe_against_local_runtime \
  -- --ignored --exact --nocapture
```

## Expected Evidence
- the ignored proof test passes for `S-04`
- the live `sdk-direct` driver executes create-task, fund-escrow, accept-task, complete-task, and release-escrow
- task and escrow identifiers are non-empty through the checked-in probe validators
- state-bearing responses remain non-empty through the checked-in probe validators

## Notes
- live task-lifecycle slice: `docs/validation/live-task-lifecycle-slice.md`
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This slice is intentionally bounded to one local-heavy `sdk-direct` S-04 lane on current `main`.
