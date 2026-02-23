# E2E Live Testing PRD R55 Live S-02 Matrix Execution Evidence

## Context
This artifact records issue `#5812` execution evidence for live harness runs that include `S-02` across:
- `sdk-direct`
- `cli-scripted`
- `mcp-tau`

Scenario set: `S-01,S-02,S-04,S-06`.

## Baseline (Before #5812)
- Existing live evidence artifacts validated `S-01,S-04,S-06` only.
- `r55_live_s02_execution_contract=missing`

## Executed Commands

```bash
mkdir -p .tmp/5812-live

# run in one persistent shell session
target/debug/kamn-node \
  --role processor \
  --runtime-mode api \
  --api-bind 127.0.0.1:8080 \
  --api-max-requests 20000 \
  --api-idle-timeout-ms 60000 \
  > .tmp/5812-live/kamn-node.log 2>&1 &
NODE_PID=$!

KAMN_E2E_SDK_DIRECT_LIVE=1 \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode sdk-direct \
  --kolme-binary /bin/true \
  --evidence-dir .tmp/5812-live/sdk-direct \
  --scenarios S-01,S-02,S-04,S-06 > .tmp/5812-live/sdk-direct.json

KAMN_E2E_CLI_SCRIPTED_LIVE=1 \
KAMN_E2E_CLI_BINARY="$(pwd)/target/debug/kamn-cli" \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode cli-scripted \
  --kolme-binary /bin/true \
  --evidence-dir .tmp/5812-live/cli-scripted \
  --scenarios S-01,S-02,S-04,S-06 > .tmp/5812-live/cli-scripted.json

KAMN_E2E_MCP_AGENT_LIVE=1 \
KAMN_E2E_MCP_AGENT_BINARY="$(pwd)/target/debug/kamn-mcp-server" \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode mcp-tau \
  --kolme-binary /bin/true \
  --agent-binary /bin/true \
  --evidence-dir .tmp/5812-live/mcp-tau \
  --scenarios S-01,S-02,S-04,S-06 > .tmp/5812-live/mcp-tau.json

kill "$NODE_PID"
```

## Observed Results
- `.tmp/5812-live/sdk-direct.json`: `S-01=PASS`, `S-02=PASS`, `S-04=PASS`, `S-06=PASS`
- `.tmp/5812-live/cli-scripted.json`: `S-01=PASS`, `S-02=PASS`, `S-04=PASS`, `S-06=PASS`
- `.tmp/5812-live/mcp-tau.json`: `S-01=PASS`, `S-02=PASS`, `S-04=PASS`, `S-06=PASS`

Node service log evidence (`.tmp/5812-live/kamn-node.log`) includes successful S-02 route execution across all three modes:
- `POST /v1/messages/send` -> `202`
- `GET /v1/messages/{message_id}` -> `200`
- Reply send/query sequence succeeds per mode with `2xx` outcomes and no auth/scope failures.

## Status Markers
- `r55_live_s02_execution_schema_version=kamn.e2e.live-s02-execution.v1`
- `r55_live_s02_execution_modes_executed=3`
- `r55_live_s02_execution_scenarios_executed_csv=S-01,S-02,S-04,S-06`
- `r55_live_s02_execution_sdk_direct_status=pass`
- `r55_live_s02_execution_cli_scripted_status=pass`
- `r55_live_s02_execution_mcp_tau_status=pass`
- `r55_live_s02_execution_s01_sdk_direct_status=pass`
- `r55_live_s02_execution_s02_sdk_direct_status=pass`
- `r55_live_s02_execution_s04_sdk_direct_status=pass`
- `r55_live_s02_execution_s06_sdk_direct_status=pass`
- `r55_live_s02_execution_s01_cli_scripted_status=pass`
- `r55_live_s02_execution_s02_cli_scripted_status=pass`
- `r55_live_s02_execution_s04_cli_scripted_status=pass`
- `r55_live_s02_execution_s06_cli_scripted_status=pass`
- `r55_live_s02_execution_s01_mcp_tau_status=pass`
- `r55_live_s02_execution_s02_mcp_tau_status=pass`
- `r55_live_s02_execution_s04_mcp_tau_status=pass`
- `r55_live_s02_execution_s06_mcp_tau_status=pass`
- `r55_live_s02_execution_overall_status=pass`
- `r55_live_s02_execution_contract=implemented`
