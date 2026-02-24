# Spec: Issue #5853 - Full-Mode Supervisor Concurrent Endpoint Lanes

- Issue: #5853
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`kamn-node` currently executes `runtime-mode full` daemon work to completion before starting configured Service API and observability endpoint lanes in the CLI runtime path. This leaves full-mode endpoint behavior structurally coupled to post-daemon execution instead of supervised alongside daemon execution.

## Scope
In scope:
- Add full-mode runtime supervision path in `kamn-node` CLI execution so configured endpoint lanes are started before daemon completion and run under explicit lifecycle handling.
- Enforce deterministic full-mode endpoint lane contracts and fail-closed reason codes for invalid lane settings.
- Add runtime tests that assert endpoint lane start markers are emitted before full supervisor stop-complete markers.
- Preserve existing full-supervisor daemon stop-contract behavior and reason-code stability.

Out of scope:
- New dependencies.
- Wire/protocol changes to service API or observability endpoint payloads.
- Changes to non-`kamn-node` crates except compilation and contract compatibility adjustments.

## Acceptance Criteria
- AC-1: In `runtime-mode full`, configured endpoint lanes are started via supervisor path before daemon execution completes.
- AC-2: Full-mode endpoint lanes use deterministic lifecycle markers and produce fail-closed `ConfigError::RuntimeDaemonLifecycle` reason codes when lane contracts are violated.
- AC-3: Existing full-supervisor stop-contract reason-code behavior remains stable.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing for the new supervisor behavior.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | full mode + `--api-bind` via CLI runtime path | log marker `node.runtime.service_api.endpoint.start` appears before `node.runtime.full.supervisor.stop.complete` |
| C-02 | AC-2 | Unit | full mode + `--api-bind` + `--api-max-requests 2` | fail-closed `ConfigError::RuntimeDaemonLifecycle` with deterministic full-supervisor service-api lane reason code |
| C-03 | AC-2 | Unit | full mode + `--observability-endpoint-bind` + `--observability-endpoint-max-requests 2` | fail-closed `ConfigError::RuntimeDaemonLifecycle` with deterministic full-supervisor observability lane reason code |
| C-04 | AC-3 | Regression | existing full-supervisor stop classifier tests | reason-code outputs remain unchanged |

## Test Mapping
- `cargo test -p kamn-node integration_runtime_full_supervisor_starts_service_api_lane_before_daemon_stop`
- `cargo test -p kamn-node regression_runtime_full_supervisor_rejects_service_api_lane_max_requests_drift`
- `cargo test -p kamn-node regression_runtime_full_supervisor_rejects_observability_lane_max_requests_drift`
- `cargo test -p kamn-node full_supervisor_stop_contract`

## Success Metrics / Observable Signals
- Full-mode runtime logs endpoint lane start markers before daemon stop-complete marker.
- Lane-contract violations fail closed with deterministic reason-code markers.
- Targeted and regression test lanes pass without changing existing full-supervisor stop taxonomy.
