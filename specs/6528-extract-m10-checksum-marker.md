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

- [ ] Deterministic M10 checksum-marker generation lives in `kamn-data-layer`
- [ ] `kamn-core` keeps behavior stable through a compatibility wrapper
- [ ] Targeted tests prove the extracted checksum-marker surface and keep core
      M10 behavior green
- [ ] Extraction wiring docs reflect the new M10 slice

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
