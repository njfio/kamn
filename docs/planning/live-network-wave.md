# Live-Network Pilot Smoke Wave (Issue #828)

This plan defines the bounded PR-fast smoke lane used to validate live-network
pilot readiness without running deep or expensive suites on every change.

## Scope

- One-command live-network smoke validation for local developer use.
- PR-fast budget guard with deterministic fail-closed behavior.
- Machine-readable smoke report artifact for pilot evidence review.

## Commands

- Makefile developer entrypoint:
  - `make smoke-live-network`
- Direct smoke runner:
  - `bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json`
- Contract lane:
  - `bash scripts/runtime/run_live_network_smoke_contract_lane.sh`

## Evidence Contract

- Report schema:
  - `kamn.runtime.live-network-smoke-report.v1`
- Required report fields:
  - `status`
  - `final_decision`
  - `elapsed_seconds`
  - `max_seconds`
  - `command_count`
  - `commands`
  - `reason_codes`

## Runtime and Cost Policy

- Default smoke budget:
  - `KAMN_LIVE_NETWORK_SMOKE_MAX_SECONDS=120`
- Contract lane ceiling:
  - `run_live_network_smoke_contract_lane.sh` enforces a 180-second upper bound.
- Regression guard:
  - budget overflow remains fail-closed with explicit reason code `runtime_budget_exceeded` (`Regression: #828`).
