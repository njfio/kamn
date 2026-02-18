# Issue #5023 Spec

- Title: Task: M7 deliver Timescale hypertables, aggregates, and billing telemetry surfaces
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M7 requires deterministic contracts for time-series telemetry ingestion,
continuous aggregate rollups, and owner billing reconciliation markers.
Current codebase has observability and runtime modules, but no dedicated M7
time-series contract surface for agent metrics, system telemetry, and billing
aggregation behaviors.

PRD mapping:
- Section 8.2 (TimescaleDB hypertables and metrics schema)
- Section 8.2.3 (system telemetry)
- Section 8.3 (continuous aggregates: hourly/daily/network summary/owner billing)
- Section 12 (observability and alerting integration expectations)
- Milestone table M7 deliverables (hypertables + aggregates + billing metrics)

## Acceptance Criteria
- AC-1: Telemetry ingest contract accepts owner/agent-scoped metric points with
  deterministic timestamp-bucket indexing and fail-closed validation.
- AC-2: Continuous aggregate contract produces deterministic hourly and daily
  rollups for agent metrics and network summary counters.
- AC-3: Owner billing contract computes deterministic daily billing usage markers
  (messages, storage bytes, queries, embeddings) from ingested telemetry.
- AC-4: Cross-owner telemetry and billing queries are denied fail-closed unless requester matches owner scope.
- AC-5: Shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M7 module in `kamn-core` for time-series telemetry ingestion, rollup aggregation,
  and owner billing daily projection contracts.
- Conformance tests for deterministic bucket rollups, billing projection stability,
  and owner-scope authorization boundaries.
- Public API exports for downstream M8+ integration lanes.

Out of scope:
- Live TimescaleDB extension DDL, SQL continuous aggregate jobs, and Prometheus wiring.
- Runtime scheduler/background refresh infrastructure.
- New dependencies, protocol/wire-format changes, or shell/python workflow additions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Ingest deterministic telemetry points for one owner/agent over multiple timestamps | Points are accepted and indexed into stable hourly/day buckets |
| C-02 | AC-1/AC-4 | Unit | Attempt invalid DID scope and cross-owner read/query | Fail-closed typed errors are returned |
| C-03 | AC-2 | Conformance | Compute hourly + daily aggregate rollups and network summary | Deterministic aggregate values and ordering are preserved |
| C-04 | AC-3 | Conformance | Build owner billing daily projection from telemetry | Billing projection fields are complete and deterministic |
| C-05 | AC-4 | Regression | Query owner billing with mismatched requester scope | Access denied with stable reason marker |
| C-06 | AC-5 | Regression | Inspect issue diff for shell/python/workflow/template files | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` conformance tests.
- M7 contracts are exported via `kamn_core` for downstream integration lanes.
- Shell-to-Rust ratio direction is improved/neutral through Rust-only changes.

## Verification
| AC | Result | Tests/Evidence |
|---|---|---|
| AC-1 | ✅ | `spec_c01_telemetry_ingest_indexes_points_into_deterministic_hourly_and_daily_buckets`, `spec_c02_invalid_scope_inputs_fail_closed` |
| AC-2 | ✅ | `spec_c03_hourly_daily_and_network_rollups_are_deterministic` |
| AC-3 | ✅ | `spec_c04_owner_billing_daily_projection_is_deterministic_and_complete` |
| AC-4 | ✅ | `spec_c02_invalid_scope_inputs_fail_closed`, `spec_c05_cross_owner_billing_query_is_denied_fail_closed` |
| AC-5 | ✅ | `git diff --name-only` shows no `scripts/**`, `.github/workflows/**`, or template-surface changes |

Executed commands:
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `cargo test -p kamn-core --test data_layer_m7_timeseries_telemetry`
- `cargo test -p kamn-core`
