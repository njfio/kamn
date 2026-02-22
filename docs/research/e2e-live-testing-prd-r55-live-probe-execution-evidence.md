# R55 Live Probe Execution Evidence (S-01/S-04/S-06)

- Date: 2026-02-22
- Issue: #5797
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Baseline commit: `30e0e7ad`

## Objective
Execute existing live probe paths for scenarios `S-01`, `S-04`, and `S-06` across:
- `sdk-direct`
- `cli-scripted`
- `mcp-any`

## Commands Executed

### Binary Prep
```bash
cargo build -p kamn-e2e-harness -p kamn-cli -p kamn-mcp-server
```

### Harness Run Matrix
```bash
# sdk-direct
KAMN_E2E_SDK_DIRECT_LIVE=1 target/debug/kamn-e2e-harness run \
  --mode sdk-direct \
  --kolme-binary /bin/true \
  --evidence-dir /tmp/kamn-e2e-live-sdk \
  --scenarios S-01,S-04,S-06 > /tmp/kamn-e2e-live-sdk.json

# cli-scripted
KAMN_E2E_CLI_SCRIPTED_LIVE=1 KAMN_E2E_CLI_BINARY="$(pwd)/target/debug/kamn-cli" \
  target/debug/kamn-e2e-harness run \
  --mode cli-scripted \
  --kolme-binary /bin/true \
  --evidence-dir /tmp/kamn-e2e-live-cli \
  --scenarios S-01,S-04,S-06 > /tmp/kamn-e2e-live-cli.json

# mcp-any
KAMN_E2E_MCP_AGENT_LIVE=1 KAMN_E2E_MCP_AGENT_BINARY="$(pwd)/target/debug/kamn-mcp-server" \
  target/debug/kamn-e2e-harness run \
  --mode mcp-any \
  --kolme-binary /bin/true \
  --agent-binary "$(pwd)/target/debug/kamn-mcp-server" \
  --evidence-dir /tmp/kamn-e2e-live-mcp \
  --scenarios S-01,S-04,S-06 > /tmp/kamn-e2e-live-mcp.json
```

## Outcome Matrix (No Local API Services)

| Mode | Overall | Validation | S-01 | S-04 | S-06 |
|---|---|---|---|---|---|
| `sdk-direct` | `FAIL` | `FAIL` | `FAIL` | `FAIL` | `PASS` |
| `cli-scripted` | `FAIL` | `FAIL` | `FAIL` | `FAIL` | `PASS` |
| `mcp-any` | `FAIL` | `FAIL` | `FAIL` | `FAIL` | `PASS` |

Parsed from:
- `/tmp/kamn-e2e-live-sdk.json`
- `/tmp/kamn-e2e-live-cli.json`
- `/tmp/kamn-e2e-live-mcp.json`

## Outcome Matrix (KAMN API Service Running on `:8080`)

Command used to host service API in this workspace:

```bash
target/debug/kamn-node \
  --role listener \
  --runtime-mode api \
  --api-bind 127.0.0.1:8080 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 60000 \
  --storage-dir /tmp/<run>/data \
  --output json
```

Live matrix outcome with this process active:

| Mode | Overall | Validation | S-01 | S-04 | S-06 |
|---|---|---|---|---|---|
| `sdk-direct` | `FAIL` | `FAIL` | `PASS` | `FAIL` | `PASS` |
| `cli-scripted` | `FAIL` | `FAIL` | `PASS` | `FAIL` | `PASS` |
| `mcp-any` | `FAIL` | `FAIL` | `PASS` | `FAIL` | `PASS` |

Parsed from the same output files:
- `/tmp/kamn-e2e-live-sdk.json`
- `/tmp/kamn-e2e-live-cli.json`
- `/tmp/kamn-e2e-live-mcp.json`

## Environment Bring-Up Attempts

### Attempt 1
```bash
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/runtime/run_live_validation_environment_lane.sh --mode run --output-json /tmp/live-validation-environment.json
```
Result: `FAIL` with bundle reason `integration_bundle_failed`.

### Attempt 2 (checkout reset)
```bash
rm -rf /tmp/kolme_fork
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/runtime/run_live_validation_environment_lane.sh --mode run --output-json /tmp/live-validation-environment.json
```
Result: `FAIL`; bootstrap reason moved to `checkout_path_missing`.

### Attempt 3 (checkout cloned)
```bash
git clone --depth 1 https://github.com/njfio/kolme_fork.git /tmp/kolme_fork
KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/runtime/run_live_validation_environment_lane.sh --mode run --output-json /tmp/live-validation-environment.json
```
Result: `FAIL`; bootstrap now passes fork metadata sync but API probe fails with `healthz_request_failed` against `http://127.0.0.1:3000/healthz`.

### Attempt 4 (run live matrix with real `kamn-node` API service)
`S-01` transitioned to `PASS` in all 3 modes once `kamn-node` served `/healthz` on `127.0.0.1:8080`.

`S-04` remained `FAIL` in all 3 modes due protected-route auth rejection on task creation (`POST /v1/tasks/create`).

### Attempt 5 (auth diagnostics against `create-task`)

With default node chain context (`kamn-devnet/v0.1.0`), CLI returns:

```text
reason_code=service_api_auth_signature_verification_failed
```

When node chain context is aligned to the SDK default signer state (`--chain-id kamn-agent-lib --chain-version 1`), the rejection shifts to:

```text
reason_code=service_api_auth_scope_header_missing
```

This confirms `S-04` is now blocked by SDK/agent-lib request auth contract drift for protected service routes.

## Reproducible Blockers

1. Kolme local endpoint (`http://127.0.0.1:3000`) is still required for live-environment lane success and is not available in this workspace.
2. `S-04` fails on real KAMN API due request-auth contract mismatch:
   - signature state-hash mismatch unless node chain context matches SDK defaults
   - missing `x-kamn-authz-scope` header for protected routes even after chain alignment
3. Until auth contracts align, live task lifecycle probes (`S-04`) fail across sdk/cli/mcp modes.

## Minimal Prerequisites to Unblock

1. Start Kolme local endpoint with healthy `GET /healthz` at `http://127.0.0.1:3000` for full live-environment lane parity.
2. Run KAMN service API on `127.0.0.1:8080` with stable request budget (`--api-max-requests 1000`).
3. Align SDK/agent-lib protected-route auth to service scope policy (`x-kamn-authz-scope`) and signature chain context.
4. Keep `target/debug/kamn-cli` and `target/debug/kamn-mcp-server` available for CLI/MCP live runners.
5. Re-run the harness matrix and verify `S-04` transitions to `PASS` in all modes.

## Current Status
`🔴 Blocked` — `S-01` is now green on a real local KAMN API, but `S-04` remains blocked by protected-route auth contract drift (`service_api_auth_signature_verification_failed` / `service_api_auth_scope_header_missing`).
