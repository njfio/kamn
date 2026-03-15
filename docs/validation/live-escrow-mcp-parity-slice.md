# Live Escrow MCP Parity Slice

This runbook documents one bounded MCP-agent parity slice for live escrow settlement on current `main`. It is anchored to the checked-in MCP-agent S-05 probe in `kamn-e2e-harness`. It proves one local-heavy MCP execution lane against a running local Kolme runtime and local KAMN API runtime. It does not prove Solana-backed settlement, bridge settlement, or external-chain settlement.

## Scope
- one real local-heavy MCP-agent S-05 probe on current `main`
- local Kolme plus local KAMN API runtime on loopback
- explicit parity proof against the existing live escrow settlement slices

## Proof Anchors
- `crates/kamn-e2e-harness/src/drivers/mcp_agent/live_probe_tranche_one/escrow_settlement_probe.rs`
- `crates/kamn-e2e-harness/tests/live_s05_mcp_agent_execution.rs`
- `docs/validation/live-escrow-settlement-slice.md`
- `docs/validation/live-escrow-cli-parity-slice.md`

## What This Proves
- the checked-in MCP-agent driver can execute S-05 escrow settlement on current `main`
- the proof is exercised through the explicit ignored integration test `integration_live_s05_mcp_agent_escrow_settlement_probe_against_local_runtime`
- current `main` has one operator-comprehensible MCP parity lane for the same bounded live escrow settlement slice already proven through `sdk-direct` and CLI-scripted drivers

## What This Does Not Prove
- does not prove Solana-backed settlement
- does not prove bridge settlement
- does not prove external-chain settlement
- does not prove bridge-backed value movement
- does not prove Byzantine-safe dispute resolution

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
  --storage-dir /tmp/kamn-node-live-mcp-s05
```

## Operator Commands
Export the live MCP parity env:

```bash
export KAMN_E2E_MCP_AGENT_LIVE=true
export KAMN_E2E_MCP_AGENT_BINARY=/home/n/Code/kamn/target/debug/kamn-mcp-server
export KAMN_ENDPOINT=http://127.0.0.1:18180
export KAMN_AGENT_NAME=kamn-live-mcp-s05-proof
export KAMN_AGENT_KEY_FILE=/tmp/kamn-live-mcp-s05.key
```

Run the explicit ignored proof test:

```bash
cargo test -p kamn-e2e-harness \
  --test live_s05_mcp_agent_execution \
  integration_live_s05_mcp_agent_escrow_settlement_probe_against_local_runtime \
  -- --ignored --exact --nocapture
```

## Expected Evidence
- the ignored proof test passes for `S-05`
- the live MCP driver invokes `probe-fund-escrow` then `probe-release-escrow`
- MCP output includes non-empty `escrow_id`
- MCP output returns settlement `state` values required by the checked-in probe validators

## Notes
- live escrow MCP parity slice: `docs/validation/live-escrow-mcp-parity-slice.md`
- The proof anchor is the explicit ignored integration test, not the harness scaffold alone.
- This slice is parity-only for the bounded live escrow settlement lane already proven through `sdk-direct` and CLI-scripted drivers.
