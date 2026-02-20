# Issue #5273 Spec

- Title: Task: implement M5 pgvector adapter projection and fail-closed extension contracts
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M5 vector contracts currently stop at in-memory registry/query decisions. The PostgreSQL adapter surface has no deterministic pgvector projection contract for M5 embedding insert/search paths, so Phase-4 extension adapters remain disconnected from runtime persistence boundaries.

## Scope
In:
- Add deterministic pgvector projection contracts in the PostgreSQL repository bridge for:
  - embedding insert projection from M5 records,
  - owner-scoped similarity search projection from M5 query inputs.
- Add fail-closed error branches for:
  - pgvector extension unavailable,
  - vector-dimension mismatch against configured pgvector dimensionality.
- Add bridge-level tests proving M5-to-adapter projection composition and fail-closed behavior.

Out:
- Live pgvector extension provisioning in CI/dev environments.
- End-to-end live SQL execution against pgvector tables.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0018
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: M5 embedding records can be projected into deterministic pgvector insert descriptors with stable bind order.
- AC-2: M5 semantic query inputs can be projected into deterministic owner-scoped pgvector similarity search descriptors.
- AC-3: Projection fails closed with stable reason markers when pgvector is unavailable or vector dimensions mismatch configured dimensionality.
- AC-4: Unit/Functional/Integration/Regression coverage for this slice passes with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | server-side M5 embedding record + enabled pgvector config | deterministic insert descriptor kind/sql/bind markers |
| C-02 | AC-2 | Functional | owner-scoped semantic query + enabled pgvector config | deterministic similarity-search descriptor kind/sql/bind markers |
| C-03 | AC-3 | Regression | pgvector disabled config | fail-closed bridge error with extension-unavailable reason code |
| C-04 | AC-3 | Regression | query/embedding vector dimensions != configured dimensions | fail-closed bridge error with dimension-mismatch reason code |
| C-05 | AC-4 | Integration | M5 registry append output projected via pgvector bridge function | projection composes coherently across M5 + PG bridge boundaries |
| C-06 | AC-4 | Verification | fmt/clippy + targeted bridge tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics
- Phase-4 story `#5251` has its first concrete runtime adapter boundary contract implemented without shell-surface growth.
- M5 embedding/search contracts can now map to deterministic pgvector SQL descriptors with explicit fail-closed guardrails.

## Verification Evidence
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge --test public_api_surface_policy` ✅
- `cargo fmt --check` ✅
- `cargo clippy -p kamn-core --tests -- -D warnings` ✅
