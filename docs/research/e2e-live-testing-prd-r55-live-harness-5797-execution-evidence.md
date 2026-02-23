# E2E Live Testing PRD R55 - Issue #5797 Live Harness Execution Evidence

## Context
This artifact records issue `#5797` execution evidence for live harness runs across:
- `sdk-direct`
- `cli-scripted`
- `mcp-tau`

Scenario set: `S-01,S-04,S-06`.

## Prerequisites
- Built binaries:
  - `target/debug/kamn-node`
  - `target/debug/kamn-cli`
  - `target/debug/kamn-mcp-server`
  - `target/debug/kamn-e2e-harness`
- Local API runtime available on `http://127.0.0.1:8080`.

## Execution Notes
- Initial matrix attempts from separate one-shot shells produced false failures (`S-01`, `S-04`) because the local API process was not preserved for subsequent commands in this execution environment.
- Final evidence run executed the node runtime and all three harness modes in a single persistent shell lifecycle.

## Executed Commands

```bash
mkdir -p .tmp/5797-live

# run in one persistent shell session
target/debug/kamn-node \
  --role processor \
  --runtime-mode api \
  --api-bind 127.0.0.1:8080 \
  --api-max-requests 20000 \
  --api-idle-timeout-ms 60000 \
  > .tmp/5797-live/kamn-node.log 2>&1 &
NODE_PID=$!

KAMN_E2E_SDK_DIRECT_LIVE=1 \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode sdk-direct \
  --kolme-binary /bin/true \
  --evidence-dir .tmp/5797-live/sdk-direct \
  --scenarios S-01,S-04,S-06 > .tmp/5797-live/sdk-direct.json

KAMN_E2E_CLI_SCRIPTED_LIVE=1 \
KAMN_E2E_CLI_BINARY="$(pwd)/target/debug/kamn-cli" \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode cli-scripted \
  --kolme-binary /bin/true \
  --evidence-dir .tmp/5797-live/cli-scripted \
  --scenarios S-01,S-04,S-06 > .tmp/5797-live/cli-scripted.json

KAMN_E2E_MCP_AGENT_LIVE=1 \
KAMN_E2E_MCP_AGENT_BINARY="$(pwd)/target/debug/kamn-mcp-server" \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode mcp-tau \
  --kolme-binary /bin/true \
  --agent-binary /bin/true \
  --evidence-dir .tmp/5797-live/mcp-tau \
  --scenarios S-01,S-04,S-06 > .tmp/5797-live/mcp-tau.json

kill "$NODE_PID"
```

## Observed Results
- `.tmp/5797-live/sdk-direct.json`: `overall=PASS`, `S-01=PASS`, `S-04=PASS`, `S-06=PASS`
- `.tmp/5797-live/cli-scripted.json`: `overall=PASS`, `S-01=PASS`, `S-04=PASS`, `S-06=PASS`
- `.tmp/5797-live/mcp-tau.json`: `overall=PASS`, `S-01=PASS`, `S-04=PASS`, `S-06=PASS`

Node service log evidence (`.tmp/5797-live/kamn-node.log`) includes successful S-04 protected-route execution across all modes:
- `POST /v1/tasks/create` -> `201`
- `POST /v1/escrow/fund` -> `200`
- `POST /v1/tasks/{id}/accept` -> `200`
- `POST /v1/tasks/{id}/complete` -> `200`
- `POST /v1/escrow/{id}/release` -> `200`

## Status Markers
- `r55_live_harness_5797_schema_version=kamn.e2e.live-harness-execution.v1`
- `r55_live_harness_5797_modes_executed=3`
- `r55_live_harness_5797_scenarios_executed_csv=S-01,S-04,S-06`
- `r55_live_harness_5797_sdk_direct_status=pass`
- `r55_live_harness_5797_cli_scripted_status=pass`
- `r55_live_harness_5797_mcp_tau_status=pass`
- `r55_live_harness_5797_s01_status=pass`
- `r55_live_harness_5797_s04_status=pass`
- `r55_live_harness_5797_s06_status=pass`
- `r55_live_harness_5797_overall_status=pass`
