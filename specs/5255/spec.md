# Issue #5255 Spec

- Title: Task: bootstrap data-layer PostgreSQL migration scaffolding and schema contract markers
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
`docs/review/data-layer-roadmap.md` identifies a contract-to-runtime gap: data-layer contracts exist, but executable PostgreSQL schema migrations do not. Without a baseline migration set, the persistence bridge cannot begin and schema assumptions can drift from contract modules.

## Scope
In:
- Add baseline SQL migration scaffolding for six core tables in `crates/kamn-core/migrations/`.
- Include deterministic index and RLS marker blocks in the migration.
- Add Rust contract tests that fail closed when required migration markers are missing.
- Update roadmap/plan/milestone docs with issue linkage and phase status.

Out:
- Live database connectivity and runtime query execution.
- Full `sqlx` repository implementation.
- Extension adapters (pgvector, AGE, Timescale) and realtime/compliance workers.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 120
- shell_to_rust_ratio_delta_estimate: -0.0006
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Baseline migration SQL exists with required table definitions: `messages`, `merkle_batches`, `did_registry`, `escrows`, `key_rotation_log`, and `access_log`.
- AC-2: Migration includes explicit deterministic marker blocks for required indexes and RLS policy templates.
- AC-3: Deterministic Rust tests fail closed when any required migration marker is removed or renamed.
- AC-4: Roadmap/plan/milestone artifacts link this execution track and report phase status without adding shell surface.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | baseline migration SQL | all six required table markers exist |
| C-02 | AC-2 | Functional | baseline migration SQL | required index and RLS markers exist |
| C-03 | AC-3 | Regression | marker contract tests | missing/renamed markers produce deterministic failures |
| C-04 | AC-4 | Conformance | roadmap/plan/milestone docs | all artifacts include `#5247/#5248/#5255` linkage and phase state |

## Test Mapping
- C-01/C-02/C-03:
  - `cargo test -p kamn-core --test data_layer_postgres_migration_contract`
- C-04:
  - `cargo test -p kamn-core --test data_layer_postgres_migration_contract roadmap`
  - doc artifact verification in review

## Success Metrics
- Migration scaffolding exists and is contract-tested.
- Deterministic fail-closed checks prevent silent schema marker drift.
- Phase-1 bootstrap is now ready for repository and RLS execution work.
