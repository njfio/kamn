# Issue #5275 Spec

- Title: Task: implement M6 AGE adapter projection and fail-closed graph-extension contracts
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M6 trust-graph contracts are currently isolated from PostgreSQL extension adapter boundaries. There is no deterministic AGE projection contract for graph-edge write/read flows, leaving Phase-4 graph integration incomplete.

## Scope
In:
- Add deterministic AGE projection contracts in PostgreSQL bridge for:
  - graph-edge write projection,
  - owner-scoped trust-propagation query projection.
- Add fail-closed branches with stable reason markers for:
  - AGE extension unavailable,
  - unsupported relation-kind projection.
- Add bridge-level tests validating deterministic projection and fail-closed branches.

Out:
- Live AGE extension installation/provisioning.
- Cross-owner or cross-cluster graph federation.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0018
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: M6 graph-edge inputs project deterministic AGE write descriptors with stable bind order.
- AC-2: M6 trust-propagation query inputs project deterministic AGE read descriptors.
- AC-3: Extension-unavailable and invalid relation-kind branches fail closed with stable reason markers.
- AC-4: Unit/Functional/Integration/Regression coverage for this slice passes with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid graph-edge write input + enabled AGE config | deterministic write descriptor kind/sql/bind markers |
| C-02 | AC-2 | Functional | valid trust-propagation query + enabled AGE config | deterministic read descriptor kind/sql/bind markers |
| C-03 | AC-3 | Regression | AGE disabled config | fail-closed bridge error with extension-unavailable reason code |
| C-04 | AC-3 | Regression | unsupported relation-kind projection input | fail-closed bridge error with relation-invalid reason code |
| C-05 | AC-4 | Integration | M6 graph contract output projected via AGE bridge API | coherent cross-module composition |
| C-06 | AC-4 | Verification | fmt/clippy + targeted bridge tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics
- Phase-4 story `#5251` advances with explicit M6 AGE bridge contracts without shell growth.
- M6 graph operations gain deterministic adapter projection coverage with fail-closed extension guardrails.

## Verification Evidence
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge --test public_api_surface_policy` ✅
- `cargo fmt --check` ✅
- `cargo clippy -p kamn-core --tests -- -D warnings` ✅
