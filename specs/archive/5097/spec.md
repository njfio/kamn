# Issue #5097 Spec

- Title: Task: integrate M7 owner telemetry with observability monitor contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M7 telemetry currently ingests and aggregates owner/agent metrics but does not project those points into canonical `observability.rs` contracts. This leaves health evaluation in a parallel path and prevents deterministic interoperability between owner telemetry and runtime SLO monitoring.

## Acceptance Criteria
- AC-1: M7 defines deterministic telemetry-point to observability-sample projection contracts.
- AC-2: M7 exposes owner-scoped observability evaluation using `ObservabilityMonitor` and returns deterministic report/snapshot outputs.
- AC-3: Cross-owner observability evaluation is denied fail-closed with stable owner-scope reason markers.
- AC-4: Existing M7 aggregate/billing contract behavior remains deterministic and backward compatible.
- AC-5: Shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs`
- `crates/kamn-core/tests/data_layer_m7_timeseries_telemetry.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5097/{spec.md,plan.md,tasks.md}`

Out of scope:
- Protocol/wire-format changes.
- New dependencies.
- M3/M10 integration gap work.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Convert valid telemetry record to observability sample | Deterministic sample mapping with stable field derivation |
| C-02 | AC-2 | Functional | Evaluate owner telemetry points through observability profile | Deterministic reports + snapshot counts/health |
| C-03 | AC-3 | Regression | Evaluate cross-owner request | Fail-closed `OwnerScopeViolation` with stable reason code |
| C-04 | AC-4 | Regression | Existing aggregate/billing suite | Existing M7 tests remain green |
| C-05 | AC-5 | Regression | Shell guardrail checks | Zero shell-surface growth and guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5097.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5097.json`

## Success Metrics
- M7 can emit observability-compatible evaluations from owner telemetry without parallel contract drift.
- Owner scope is enforced fail-closed for observability evaluation paths.
- Shell-to-Rust governance posture remains improved/neutral with zero shell delta.
