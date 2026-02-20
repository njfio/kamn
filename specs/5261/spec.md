# Issue #5261 Spec

- Title: Task: extend PostgreSQL execution adapter with RLS policy application and blind-index search execution
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-1 live adapter coverage (`#5259`) shipped migration + insert/select execution paths, but runtime infrastructure is still missing blind-index search execution and deterministic M2 RLS statement application.

## Scope
In:
- Add blind-index search execution API in `DataLayerPgExecutionAdapter` using projected descriptors.
- Add deterministic default RLS statement application API using `data_layer_pg_project_default_rls_statements`.
- Add structured reports for search and RLS execution output.
- Add conformance tests for success/failure paths.

Out:
- pgvector/AGE/Timescale extension execution.
- Envelope crypto pipeline and Kolme anchoring runtime.
- Cross-cluster or multi-tenant policy orchestration beyond default M2 templates.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 280
- shell_to_rust_ratio_delta_estimate: -0.0015
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Adapter executes blind-index search descriptors with requester DID session context and deterministic result decoding.
- AC-2: Adapter applies default M2 RLS statements in deterministic order and returns structured execution report.
- AC-3: SQL/session/policy application failures return structured non-panicking errors.
- AC-4: Targeted adapter, bridge, and policy suites pass with fmt/clippy gates.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid blind-index search request + requester DID | deterministic rows returned in descriptor order |
| C-02 | AC-2 | Integration | default projected RLS statements | statements execute in deterministic order and report row counts |
| C-03 | AC-3 | Regression | invalid requester/session + SQL failure | fail-closed structured error variants |
| C-04 | AC-4 | Verification | fmt/clippy + adapter suites | all required checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge`
- `cargo test -p kamn-core --test public_api_surface_policy`

## Success Metrics
- Phase-1 runtime adapter supports insert/select/search and RLS policy application primitives.
- Story `#5248` closure risk is reduced by eliminating remaining live-execution gaps.
