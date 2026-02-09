# Live-Network Pilot Validation Wave (Issues #828 / #829)

This plan defines the bounded PR-fast smoke lane plus the scheduled/manual deep
lane used to validate live-network pilot readiness while keeping PR cost low.

## Scope

- One-command live-network smoke validation for local developer use.
- Scheduled/manual live-network deep validation lane for pilot evidence review.
- PR-fast budget guard with deterministic fail-closed behavior.
- Machine-readable smoke report and pilot artifact summary contracts.

## Commands

- Makefile developer entrypoints:
  - `make smoke-live-network`
  - `make deep-live-network`
  - `make demo-localhost-transport`
- Direct smoke runner:
  - `bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json`
- Smoke contract lane:
  - `bash scripts/runtime/run_live_network_smoke_contract_lane.sh`
- Scheduled/manual deep lane:
  - `bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name schedule --output-json /tmp/live-network-pilot-report.json`
- Deep contract lane:
  - `bash scripts/runtime/run_live_network_pilot_deep_contract_lane.sh`
- Deep summary policy checker:
  - `bash scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh --summary-file /tmp/live-network-pilot-report.json`
- Localhost signed sender/listener transport demo:
  - `bash scripts/sdk/run_localhost_signed_demo.sh`

## Bridge Replay/Redaction Lane Matrix

- PR-fast bridge evidence lane (bounded):
  - `bash scripts/bridge/run_bridge_replay_redaction_contract_lane.sh --skip-replay --replay-report-file bridge-replay-report.json`
- PR-fast localhost bridge demo evidence lane (bounded):
  - `bash scripts/bridge/run_localhost_bridge_demo_evidence_contract_lane.sh`
- Scheduled/manual localhost bridge demo evidence lane (deep):
  - `bash scripts/bridge/run_localhost_bridge_demo_evidence_deep_lane.sh --output-json /tmp/localhost-bridge-demo-evidence-deep-report.json`
- Scheduled/manual bridge evidence lane (deep):
  - `bash scripts/bridge/run_bridge_replay_redaction_deep_lane.sh --output-json /tmp/bridge-replay-redaction-deep-report.json`
- Bridge evidence bundle policy checker:
  - `bash scripts/bridge/check_bridge_replay_redaction_policy.sh --bundle-file /tmp/bridge-replay-redaction-deep-report.json`
- Localhost bridge demo evidence policy checker:
  - `bash scripts/bridge/check_localhost_bridge_demo_policy.sh --bundle-file /tmp/localhost-bridge-demo-evidence.json`

## Evidence Contract

- Smoke report schema:
  - `kamn.runtime.live-network-smoke-report.v1`
- Pilot summary schema:
  - `kamn.runtime.live-network-pilot-artifact-summary.v1`
- Localhost bridge demo evidence schema:
  - `kamn.bridge.localhost-demo-evidence.v1`
- Required smoke report fields:
  - `status`
  - `final_decision`
  - `elapsed_seconds`
  - `max_seconds`
  - `command_count`
  - `commands`
  - `reason_codes`
- Required pilot summary fields:
  - `event_name`
  - `cadence`
  - `smoke`
  - `deep`
  - `budget_status`
  - `evidence_complete`
  - `decision_reasons`
  - `final_decision`

## Runtime and Cost Policy

- Default smoke budget:
  - `KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS=120`
- Default deep lane budget:
  - `KAMN_LIVE_NETWORK_PILOT_DEEP_MAX_SECONDS=300`
- Smoke contract lane ceiling:
  - `run_live_network_smoke_contract_lane.sh` enforces a 180-second upper bound.
- Deep cadence policy:
  - `run_live_network_pilot_deep_lane.sh` rejects non-`schedule` and non-`workflow_dispatch` events.
- Bridge PR-fast budget:
  - `run_bridge_replay_redaction_contract_lane.sh` enforces a 120-second upper bound.
- Localhost bridge demo evidence budget:
  - `run_localhost_bridge_demo_evidence_contract_lane.sh` enforces a 120-second upper bound.
- Localhost bridge demo evidence deep budget:
  - `run_localhost_bridge_demo_evidence_deep_lane.sh` enforces a 300-second upper bound.
- Bridge deep lane budget:
  - `run_bridge_replay_redaction_deep_lane.sh` enforces a 300-second upper bound.
- Dashboard runtime compatibility guard:
  - `bash scripts/frontend/test_dashboard_package_runtime_compat.sh` validates fallback behavior when local `node` lacks `--experimental-strip-types`.
  - `scripts/frontend/test_dashboard_package.sh` defaults fallback execution to `npx -y node@22` in fail-closed mode.
- Regression guard:
  - budget overflow remains fail-closed with explicit reason code `runtime_budget_exceeded` (`Regression: #828`).
- Regression guard:
  - tampered pilot summary `final_decision` is rejected by policy checker (`Regression: #829`).
- Regression guard:
  - dashboard runtime fallback contract remains pinned to `node@22` with local reproduction guidance (`Regression: #868`).
