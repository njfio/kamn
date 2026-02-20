# Issue #5259 Spec

- Title: Task: implement sqlx-backed PostgreSQL execution adapter and migration runner
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
Phase-1 now has migration artifacts (`#5255`) and deterministic repository bridge contracts (`#5257`), but lacks a live execution layer to apply migrations and execute projected SQL operations against PostgreSQL with requester DID RLS session context.

## Scope
In:
- Add `sqlx`-backed PostgreSQL execution adapter in `kamn-core`.
- Add migration runner integration for `crates/kamn-core/migrations/`.
- Execute bridge-projected insert/lookup operations in transactions.
- Apply requester DID session setting before RLS-sensitive operations.

Out:
- pgvector/AGE/Timescale extension adapters.
- Realtime delivery and compliance job execution.
- Cross-cluster orchestration.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 420
- shell_to_rust_ratio_delta_estimate: -0.0019
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Live adapter executes insert and lookup operations projected by `data_layer_postgres_repository_bridge`.
- AC-2: Migration runner applies baseline migration artifacts in deterministic order.
- AC-3: Requester DID session setting (`kamn.requester_did`) is set before executing RLS-governed operations.
- AC-4: Invalid session context and SQL execution failures return structured, non-panicking errors.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid insert/lookup descriptors | rows persist and resolve through adapter |
| C-02 | AC-2 | Integration | migration set from `migrations/` | deterministic migration application order |
| C-03 | AC-3 | Integration | requester DID session context | setting applied before query and RLS-compatible |
| C-04 | AC-4 | Regression | invalid DID/session + SQL failures | structured fail-closed error variants |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test data_layer_postgres_migration_contract`
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge`

## Success Metrics
- Adapter can execute bridge descriptors against PostgreSQL.
- Migration runner and session-context setup are deterministic and test-validated.
- Phase-1 story `#5248` advances from contract scaffolding to live execution.
