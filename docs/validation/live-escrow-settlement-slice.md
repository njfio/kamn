# Live Escrow Settlement Slice

This runbook documents one bounded live escrow settlement slice on current `main`. It is anchored to external-execution `sdk-direct` S-05 on the checked-in `kamn-e2e-harness` surface. It proves one live escrow settlement execution lane through the existing harness driver path. It does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.

## Scope
- one real external-execution `sdk-direct` S-05 harness run on current `main`
- local Kolme plus three local KAMN API nodes on loopback
- one `verify` pass over the generated evidence bundle

## Proof Anchors
- `crates/kamn-e2e-harness/src/scenarios/s05_escrow.rs`
- `crates/kamn-e2e-harness/src/drivers/sdk_direct/live_probe_tranche_one/escrow_probe_support.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted/live_probe_tranche_one/escrow_probe_support.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent/live_probe_tranche_one/escrow_settlement_probe.rs`
- `crates/kamn-e2e-harness/tests/live_s05_sdk_direct_external_execution.rs`

## What This Proves
- the external-execution `sdk-direct` driver can execute S-05 escrow settlement on current `main`
- the run is exercised through the real harness `run` command with `--enable-external-execution`
- the generated evidence bundle can be checked by the real harness `verify` command
- current `main` has one operator-comprehensible live escrow settlement slice beyond the earlier service-api persistence-only escrow proof

## What This Does Not Prove
- does not prove Solana-backed settlement
- does not prove bridge settlement
- does not prove external-chain settlement
- does not prove Byzantine-safe dispute resolution
- does not prove production readiness across all drivers
- CLI-scripted and MCP-agent parity probes exist, but this slice records `sdk-direct` execution evidence only unless separately run

## Local Setup
Build the required KAMN binaries:

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
/tmp/kolme/target/release/example-p2p api-server --bind 127.0.0.1:3000
```

Start three local KAMN nodes in separate shells:

```bash
target/debug/kamn-node \
  --runtime-mode api \
  --role processor \
  --api-bind 127.0.0.1:8080 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-processor
```

```bash
target/debug/kamn-node \
  --runtime-mode api \
  --role listener \
  --api-bind 127.0.0.1:8081 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-listener
```

```bash
target/debug/kamn-node \
  --runtime-mode api \
  --role approver \
  --api-bind 127.0.0.1:8082 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-approver
```

## Operator Commands
Enable the live `sdk-direct` lane and run the explicit local-heavy S-05 probe:

```bash
export KAMN_E2E_SDK_DIRECT_LIVE=true
export KAMN_ENDPOINT=http://127.0.0.1:18080
export KAMN_KOLME_ENDPOINT=http://127.0.0.1:13000
export KAMN_AGENT_NAME=kamn-live-s05-proof
export KAMN_E2E_EXTERNAL_KAMN_PROCESSOR_BINARY=/home/n/Code/kamn/target/debug/kamn-node
export KAMN_E2E_EXTERNAL_KAMN_LISTENER_BINARY=/home/n/Code/kamn/target/debug/kamn-node
export KAMN_E2E_EXTERNAL_KAMN_APPROVER_BINARY=/home/n/Code/kamn/target/debug/kamn-node

cargo test -p kamn-e2e-harness \
  --test live_s05_sdk_direct_external_execution \
  integration_live_s05_sdk_direct_escrow_settlement_probe_against_local_runtime \
  -- --ignored --exact --nocapture
```

Optional harness external-execution contract run for the surrounding orchestration surface:

```bash
target/debug/kamn-e2e-harness run \
  --mode sdk-direct \
  --kolme-binary /tmp/kolme/target/release/example-p2p \
  --enable-external-execution \
  --evidence-dir /tmp/kamn-e2e-live-s05-evidence \
  --scenarios S-05 > /tmp/kamn-e2e-live-s05-run.json
```

Verify the generated evidence contract output:

```bash
target/debug/kamn-e2e-harness verify \
  --evidence-dir /tmp/kamn-e2e-live-s05-evidence \
  --kolme-chain-dump /tmp/kamn-e2e-live-s05-evidence/kolme_chain_dump.json \
  --output /tmp/kamn-e2e-live-s05-verify-report.json > /tmp/kamn-e2e-live-s05-verify.json
```

## Expected Evidence
- explicit live probe test passes for `S-05`
- the explicit live probe uses the real `sdk-direct` driver against the running local endpoints
- optional harness `run` output records exactly one selected scenario: `S-05`
- optional harness `run` output records `S-05` status as `PASS`
- evidence bundle contains the scenario-declared artifacts:
  - `evidence/s05/escrow_lifecycle_trace.json`
  - `evidence/s05/settlement_breakdown.json`
  - `evidence/s05/dispute_resolution.json`
  - `evidence/s05/kolme_escrow_anchors.json`
- verify output completes without fail-closed evidence errors

## Notes
- live escrow settlement slice: `docs/validation/live-escrow-settlement-slice.md`
- This slice is intentionally bounded to external-execution `sdk-direct` S-05 on current `main`.
- The explicit proof command is the ignored local-heavy integration test, not the harness scaffold alone.
- It does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.
