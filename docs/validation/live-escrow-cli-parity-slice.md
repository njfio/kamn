# Live Escrow CLI Parity Slice

This runbook documents one bounded CLI-scripted parity slice for live escrow settlement on current `main`. It is anchored to the checked-in CLI-scripted S-05 probe in `kamn-e2e-harness`. It proves one local-heavy CLI execution lane against a running local Kolme runtime and local KAMN API runtime. It does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.

## Scope
- one real local-heavy CLI-scripted S-05 probe on current `main`
- local Kolme plus local KAMN API runtime on loopback
- explicit parity proof against the existing live escrow settlement slice

## Proof Anchors
- `crates/kamn-e2e-harness/src/drivers/cli_scripted/live_probe_tranche_one/escrow_probe_support.rs`
- `crates/kamn-e2e-harness/tests/live_s05_cli_scripted_execution.rs`
- `docs/validation/live-escrow-settlement-slice.md`

## What This Proves
- the checked-in CLI-scripted driver can execute S-05 escrow settlement on current `main`
- the proof is exercised through the explicit ignored integration test `integration_live_s05_cli_scripted_escrow_settlement_probe_against_local_runtime`
- current `main` has one operator-comprehensible CLI parity lane for the same bounded live escrow settlement slice already proven through `sdk-direct`

## What This Does Not Prove
- does not prove Solana-backed settlement
- does not prove bridge settlement
- does not prove external-chain settlement
- does not prove bridge-backed value movement
- does not prove Byzantine-safe dispute resolution

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
/tmp/kolme/target/release/example-p2p api-server --bind 127.0.0.1:13000
```

Start one local KAMN API node:

```bash
KAMN_SERVICE_API_TLS_MODE=disabled target/debug/kamn-node \
  --runtime-mode api \
  --role processor \
  --api-bind 127.0.0.1:18080 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-live-cli-s05
```

## Operator Commands
Export the live CLI parity env:

```bash
export KAMN_E2E_CLI_SCRIPTED_LIVE=true
export KAMN_E2E_CLI_BINARY=/home/n/Code/kamn/target/debug/kamn-cli
export KAMN_ENDPOINT=http://127.0.0.1:18080
export KAMN_AGENT_NAME=kamn-live-cli-s05-proof
```

Run the explicit ignored proof test:

```bash
cargo test -p kamn-e2e-harness \
  --test live_s05_cli_scripted_execution \
  integration_live_s05_cli_scripted_escrow_settlement_probe_against_local_runtime \
  -- --ignored --exact --nocapture
```

## Expected Evidence
- the ignored proof test passes for `S-05`
- the live CLI driver invokes `fund-escrow` then `release-escrow`
- CLI output includes non-empty `escrow_id`
- CLI output returns settlement `state` values required by the checked-in probe validators

## Notes
- live escrow CLI parity slice: `docs/validation/live-escrow-cli-parity-slice.md`
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This slice is parity-only for the bounded live escrow settlement lane already proven through `sdk-direct`.
