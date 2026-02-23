# E2E Live Testing PRD R55 Live Probe Execution Evidence

## Context
This artifact records live probe execution evidence for issue `#5799` after auth scope alignment and S-04 replay/anti-spam mitigation updates.

## Baseline (Before #5799)
- `r55_live_probe_execution_s04_sdk_direct=fail`
- `r55_live_probe_execution_s04_cli_scripted=fail`
- `r55_live_probe_execution_s04_mcp_tau=fail`
- `r55_live_probe_execution_auth_scope_contract=missing`
- `r55_live_probe_execution_chain_context_alignment=missing`

## Implemented in #5799
- SDK request-auth supports optional `x-kamn-authz-scope` header on HTTP and websocket request paths.
- Agent-lib protected routes now emit policy-correct scope markers and support chain context overrides via:
  - `KAMN_AGENT_CHAIN_ID`
  - `KAMN_AGENT_CHAIN_VERSION`
- S-04 live probe execution now uses per-step agent identities in all three drivers (SDK-direct, CLI-scripted, MCP-agent) to avoid replay/anti-spam collisions.

## Execution Commands

```bash
# local API runtime used for live probes
target/debug/kamn-node \
  --role processor \
  --runtime-mode api \
  --api-bind 127.0.0.1:8080 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 60000

# sdk-direct
KAMN_E2E_SDK_DIRECT_LIVE=1 \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode sdk-direct \
  --kolme-binary /bin/true \
  --evidence-dir .tmp/5799-live/sdk-direct \
  --scenarios S-01,S-04,S-06 > .tmp/5799-live/sdk-direct.json

# cli-scripted
KAMN_E2E_CLI_SCRIPTED_LIVE=1 \
KAMN_E2E_CLI_BINARY="$(pwd)/target/debug/kamn-cli" \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode cli-scripted \
  --kolme-binary /bin/true \
  --evidence-dir .tmp/5799-live/cli-scripted \
  --scenarios S-01,S-04,S-06 > .tmp/5799-live/cli-scripted.json

# mcp-tau
KAMN_E2E_MCP_AGENT_LIVE=1 \
KAMN_E2E_MCP_AGENT_BINARY="$(pwd)/target/debug/kamn-mcp-server" \
KAMN_ENDPOINT=http://127.0.0.1:8080 \
KAMN_AGENT_CHAIN_ID=kamn-devnet \
KAMN_AGENT_CHAIN_VERSION=v0.1.0 \
cargo run -p kamn-e2e-harness -- run \
  --mode mcp-tau \
  --kolme-binary /bin/true \
  --agent-binary /bin/true \
  --evidence-dir .tmp/5799-live/mcp-tau \
  --scenarios S-01,S-04,S-06 > .tmp/5799-live/mcp-tau.json
```

## Observed Results
- `.tmp/5799-live/sdk-direct.json`: `S-01=PASS`, `S-04=PASS`, `S-06=PASS`
- `.tmp/5799-live/cli-scripted.json`: `S-01=PASS`, `S-04=PASS`, `S-06=PASS`
- `.tmp/5799-live/mcp-tau.json`: `S-01=PASS`, `S-04=PASS`, `S-06=PASS`
- Node request logs include successful S-04 endpoints across all modes (`create`, `fund`, `accept`, `complete`, `release`) with `2xx/201` outcomes and no replay/auth scope/signature failures.

## Status Markers (After #5799)
- `r55_live_probe_execution_auth_scope_contract=implemented`
- `r55_live_probe_execution_chain_context_alignment=implemented`
- `r55_live_probe_execution_s04_sdk_direct=pass`
- `r55_live_probe_execution_s04_cli_scripted=pass`
- `r55_live_probe_execution_s04_mcp_tau=pass`
- `r55_live_probe_execution_s06_sdk_direct=pass`
- `r55_live_probe_execution_s06_cli_scripted=pass`
- `r55_live_probe_execution_s06_mcp_tau=pass`
- `r55_live_probe_execution_contract=implemented`
