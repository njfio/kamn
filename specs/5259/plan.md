# Issue #5259 Plan

- Issue: #5259
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Add `sqlx` PostgreSQL dependency wiring for `kamn-core`.
2. Implement a migration runner that executes SQL files in `crates/kamn-core/migrations/`.
3. Implement execution adapter functions that accept bridge descriptors and run statements in a transaction.
4. Ensure requester DID session key setup precedes RLS-governed SQL operations.
5. Add integration tests for migration + insert/lookup execution with failure-path coverage.

## Affected Areas
- `crates/kamn-core/Cargo.toml`
- `crates/kamn-core/src/data_layer_postgres_execution_adapter.rs` (new)
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs` (new)
- `specs/5259/{spec.md,plan.md,tasks.md}`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`

## Risks and Mitigations
- Risk: dependency changes introduce ecosystem instability.
  - Mitigation: pin explicit `sqlx` feature set and keep adapter interface narrow.
- Risk: flaky integration tests from external DB lifecycle.
  - Mitigation: deterministic setup/teardown and bounded local test fixtures.
- Risk: RLS session context ordering bugs.
  - Mitigation: explicit session-setting function and integration assertions before query execution.

## Interfaces / Contracts
- Adapter consumes `DataLayerPgSqlOperation` descriptors and returns structured execution results.
- Session context key remains `kamn.requester_did`.
- Errors remain deterministic and non-panicking.

## ADR
Not required yet; adapter integration follows existing Phase-1 roadmap design.
