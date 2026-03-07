# 6530 Extract M10 Registry State Machine

## Objective
Extract the deterministic M10 partition registry lifecycle state machine from `kamn-core` into
`kamn-data-layer` while preserving the existing `kamn-core` public API through compatibility
wrappers.

## Inputs/Outputs
- Inputs:
  - `DataLayerM10PartitionRecordInput`
  - `DataLayerM10ArchiveDueRequest`
  - partition lookup strings for reattach and recovery-readiness evaluation
  - reference month ids and planning window counts
- Outputs:
  - `DataLayerM10PartitionRecord`
  - `Vec<String>` planned partition names
  - `Vec<DataLayerM10ArchivalIndexEntry>` archival index entries
  - `DataLayerM10RecoveryReadinessReport`
  - `Vec<DataLayerM10RecoveryReadinessReport>`
  - typed lifecycle errors surfaced through the existing `kamn-core` error contract

## Boundaries/Non-goals
- In scope:
  - extract deterministic registry state-machine behavior for:
    - `register_partition`
    - `plan_future_partition_names`
    - `archive_due_partitions`
    - `reattach_partition`
    - `evaluate_partition_recovery_readiness`
    - `list_historical_recovery_readiness`
    - `project_partition_recovery_readiness`
  - add the extracted module and exports in `kamn-data-layer`
  - preserve `kamn-core::DataLayerM10PartitionLifecycleRegistry` behavior through compatibility
    delegation and type/error mapping
  - update extraction architecture docs and contract tests with a dedicated
    `m10_partition_registry_state_machine_*` slice
- Out of scope:
  - M8 compliance projection ports and projection behavior
  - `KamnDid` parsing, normalization, or DID boundary changes
  - Phase-6 orchestration, scheduler, or runtime evidence behavior
  - changing public API names or reason-code values
  - adding dependencies

## Failure Modes
- invalid `partition_month_id` fails closed with the existing invalid-month error
- duplicate partition registration fails closed with the existing duplicate error
- empty `object_storage_prefix` or `partition_name` fails closed with the existing empty-field error
- reattach for unknown partitions fails closed with the existing not-found error
- reattach from a non-archived lifecycle state fails closed with the existing invalid-transition
  error and reason code
- recovery-readiness lookup for unknown partitions fails closed with the existing not-found error
- archive planning and archival evaluation preserve existing month arithmetic validation failures

## Acceptance Criteria
- [ ] `kamn-data-layer` exposes a dedicated M10 registry lifecycle module covering deterministic
      registration, planning, archival, reattach, and recovery-readiness behavior
- [ ] `kamn-core::DataLayerM10PartitionLifecycleRegistry` keeps its existing public API and returns
      behaviorally equivalent results through compatibility delegation for the extracted methods
- [ ] extracted behavior remains deterministic for sort order, partition naming, archival metadata,
      and recovery-readiness decisions
- [ ] error behavior and stable reason codes remain unchanged at the `kamn-core` boundary for the
      extracted methods
- [ ] extraction docs and contract tests record the new registry state-machine slice and wrapper
      path markers
- [ ] targeted `kamn-data-layer` and `kamn-core` tests cover both happy-path and fail-closed cases

## Files to touch
- `specs/6530-extract-m10-registry-state-machine.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m10_partition_registry_state_machine/mod.rs`
- `crates/kamn-data-layer/src/data_layer_m10_partition_registry_state_machine/types.rs`
- `crates/kamn-data-layer/src/data_layer_m10_partition_registry_state_machine/error.rs`
- `crates/kamn-data-layer/src/data_layer_m10_partition_registry_state_machine/helpers.rs`
- `crates/kamn-data-layer/src/data_layer_m10_partition_registry_state_machine/machine.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_partition_registry_state_machine_integration.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_recoverability.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- `kamn-data-layer` should use typed lifecycle errors for the extracted state-machine surface.
- `kamn-core` must preserve the current public `DataLayerM10PartitionLifecycleError` variants and
  reason-code strings at the compatibility boundary.
- No silent normalization, fallback, or lossy error conversion is allowed.
- Entrypoint behavior does not change in this issue; only the ownership boundary changes.

## Test Plan
- Red:
  - add `kamn-data-layer` integration coverage proving the new registry state-machine module and
    exports exist and provide deterministic lifecycle behavior
  - add/update `kamn-core` coverage to prove compatibility wrappers preserve current lifecycle and
    recovery-readiness behavior
  - add/update extraction docs contract coverage for the new slice markers
- Green:
  - extract the deterministic state machine into `kamn-data-layer`
  - add core compatibility wrappers and type/error mapping
- Refactor:
  - remove duplicated registry-only deterministic logic from `kamn-core`
  - keep M8/DID/phase6 seams in `kamn-core`
- Integration:
  - run targeted `kamn-data-layer` and `kamn-core` tests
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates

## Deviations
- The extracted data-layer module shipped as a small module directory
  (`mod.rs` + `types.rs` + `error.rs` + `helpers.rs` + `machine.rs`) instead of a single
  `data_layer_m10_partition_registry_state_machine.rs` file so the new surface stays within the
  repo file-size standard.
