## Objective

Extract the deterministic M10 checksum-marker helper from `kamn-core` into
`kamn-data-layer`, while preserving current archival behavior in `kamn-core`
through a compatibility wrapper.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs`
  - `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`
  - `crates/kamn-data-layer/src/data_layer_m10_partition_month_policy.rs`
- Outputs:
  - checksum-marker generation owned by `kamn-data-layer`
  - unchanged `kamn-core` archival outcomes via delegation
  - updated extraction wiring docs/contracts for the new M10 slice

## Boundaries/Non-goals

- No M8 compliance lifecycle changes
- No DID-boundary extraction changes
- No broader M10 rewrite beyond deterministic checksum-marker policy
- No CI/workflow changes
- No change to checksum-marker bytes or hash algorithm label

## Failure modes

- Extracted checksum-marker generation produces a different digest than the
  current `kamn-core` helper
- `kamn-core` archival index generation drifts after delegation
- The extraction wiring docs fail to record the new M10 slice
- The change pulls in unrelated M10/M8/DID behavior instead of staying within
  the pure checksum-marker helper

## Acceptance criteria

- [x] Deterministic M10 checksum-marker generation lives in `kamn-data-layer`
- [x] `kamn-core` keeps behavior stable through a compatibility wrapper
- [x] Targeted tests prove the extracted checksum-marker surface and keep core
      M10 behavior green
- [x] Extraction wiring docs reflect the new M10 slice

## Files to touch

- `specs/6528-extract-m10-checksum-marker.md`
- `crates/kamn-data-layer/src/data_layer_m10_partition_month_policy.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_partition_month_policy_integration.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`

## Error semantics

- No new runtime error surface is introduced
- `kamn-core` archival behavior must remain deterministic and fail-closed as it
  does today
- The checksum-marker helper must not silently change canonical payload format
  or hash prefix

## Test plan

1. Add a red test expecting the extracted checksum-marker surface from
   `kamn-data-layer` before it exists.
2. Implement the helper in `kamn-data-layer` and delegate from `kamn-core`.
3. Run targeted `kamn-data-layer`, `kamn-core` M10, and extraction-doc tests.

## Deviations

- No deviations. The issue stayed within the pure checksum-marker helper and did
  not touch M8 or DID-boundary logic.

## Execution Evidence

- Red:
  - `cargo test -p kamn-data-layer --test data_layer_m10_partition_month_policy_integration -- --nocapture`
- Green:
  - `cargo test -p kamn-data-layer --test data_layer_m10_partition_month_policy_integration -- --nocapture`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival spec_c02_archival_due_selection_respects_retention_window_and_shred_completeness -- --exact --nocapture`
  - `cargo test -p kamn-core --lib data_layer_m10_partition_archival::shared::tests::unit_deterministic_checksum_marker_has_stable_shape -- --exact --nocapture`
- Refactor / Integration:
  - `cargo fmt --all --check`
  - `cargo test -p kamn-data-layer -- --nocapture`
  - `cargo test -p kamn-core --test data_layer_m10_partition_archival -- --nocapture`
  - `cargo test -p kamn-core --test data_layer_m0_m11_extraction_docs -- --nocapture`
  - `cargo test -p kamn-core --test test_file_size_policy -- --nocapture`
  - `cargo clippy -p kamn-data-layer --tests -- -D warnings`
  - `cargo clippy -p kamn-core --tests -- -D warnings`
