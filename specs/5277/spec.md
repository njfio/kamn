# Issue #5277 Spec

- Title: Task: implement M7 Timescale adapter projection and fail-closed telemetry-extension contracts
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M7 telemetry contracts currently have no deterministic Timescale projection boundary in the PostgreSQL bridge. Ingest and owner rollup contract outputs remain disconnected from extension-backed runtime adapter descriptors.

## Scope
In:
- Add deterministic Timescale bridge projection contracts for:
  - telemetry ingest write projection,
  - owner-scoped rollup query projection.
- Add fail-closed branches with stable reason markers for:
  - Timescale extension unavailable,
  - invalid rollup bucket-window inputs.
- Add bridge-level tests validating deterministic projection and fail-closed branches.

Out:
- Live Timescale extension provisioning/hypertable migration automation.
- Multi-owner analytics federation and cross-cluster rollup queries.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0018
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: M7 telemetry point records project deterministic Timescale ingest SQL descriptors.
- AC-2: M7 owner rollup requests project deterministic Timescale rollup query descriptors.
- AC-3: Extension-unavailable and invalid bucket-window branches fail closed with stable reason markers.
- AC-4: Unit/Functional/Integration/Regression coverage for this slice passes with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid M7 telemetry point + enabled Timescale config | deterministic ingest descriptor kind/sql/bind markers |
| C-02 | AC-2 | Functional | valid owner rollup query + enabled Timescale config | deterministic rollup descriptor kind/sql/bind markers |
| C-03 | AC-3 | Regression | Timescale disabled config | fail-closed bridge error with extension-unavailable reason code |
| C-04 | AC-3 | Regression | invalid bucket-window input | fail-closed bridge error with invalid-bucket-window reason code |
| C-05 | AC-4 | Integration | M7 registry ingest output composed into Timescale bridge ingest descriptor | coherent M7-to-bridge projection path |
| C-06 | AC-4 | Verification | fmt/clippy + targeted bridge tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_postgres_repository_bridge`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics
- Phase-4 story `#5251` completes its extension-adapter trilogy with M7 Timescale bridge contracts.
- M7 telemetry ingest/rollup pathways gain deterministic extension projection coverage without shell-surface growth.
