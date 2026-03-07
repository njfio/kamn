## Objective

Extract the deterministic M10 partition month-id and partition-name policy from
`kamn-core` into `kamn-data-layer`, while preserving the existing
`kamn-core::data_layer_m10_format_partition_name` API through a compatibility
wrapper.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs`
  - `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`
  - `crates/kamn-data-layer/src/lib.rs`
- Outputs:
  - a new `kamn-data-layer` module owning M10 month-id parsing/formatting
    policy
  - `kamn-core` wrappers/delegation preserving current behavior and exported
    signatures
  - tests proving the extracted policy in `kamn-data-layer` and unchanged
    behavior through `kamn-core`

## Boundaries/Non-goals

- No full M10 extraction
- No `data_layer_m8_compliance_lifecycle` changes
- No DID-boundary extraction changes
- No CI/workflow changes
- No public API rename for `data_layer_m10_format_partition_name`

## Failure modes

- Extracted month-id validation accepts invalid `YYYYMM` values or rejects valid
  ones
- `kamn-core::data_layer_m10_format_partition_name` behavior drifts after
  delegation
- The extraction pulls in M8/DID-boundary concerns instead of staying within
  deterministic month-id policy
- Error mapping between `kamn-data-layer` and `kamn-core` becomes lossy or
  changes error variants

## Acceptance criteria

- [ ] Deterministic M10 month-id helpers live in `kamn-data-layer`
- [ ] `kamn-core::data_layer_m10_format_partition_name` remains stable and
      delegates through a compatibility wrapper
- [ ] `kamn-data-layer` has targeted tests for the extracted month-id/partition
      policy
- [ ] Existing `kamn-core` M10 behavior remains green through targeted tests
- [ ] This issue stays scoped away from the documented M8 and DID blockers

## Files to touch

- `specs/6526-extract-m10-partition-naming-policy.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/...` new M10 partition month policy module
- `crates/kamn-data-layer/tests/...` targeted M10 partition month policy tests
- `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`

## Error semantics

- Invalid partition month ids must remain hard-fail with explicit typed errors
- `kamn-core` must preserve existing `DataLayerM10PartitionLifecycleError`
  behavior at its boundary
- No silent fallback or normalization is allowed for invalid month-id input

## Test plan

1. Add red tests proving the extracted `kamn-data-layer` module/surface is not
   yet present or not yet used.
2. Add targeted `kamn-data-layer` integration coverage for month-id parsing,
   arithmetic, and partition-name formatting.
3. Update `kamn-core` delegation and rerun targeted M10 tests to confirm public
   behavior is unchanged.
