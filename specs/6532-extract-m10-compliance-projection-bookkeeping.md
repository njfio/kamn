# 6532 Extract M10 Compliance Projection Bookkeeping

## Objective
Extract the deterministic M10 shred-completeness projection bookkeeping from
`kamn-core::data_layer_m10_partition_archival::registry` into `kamn-data-layer` while preserving
the existing `kamn-core` public projection behavior through compatibility delegation.

## Inputs/Outputs
- Inputs:
  - requester owner DID string
  - target owner DID string
  - partition month id (`YYYYMM`)
  - partition message id list
  - `DataLayerM10ComplianceProjectionPort`
  - extracted M10 partition registry state machine
- Outputs:
  - total partition message count
  - shredded partition message count
  - all-messages-shredded marker
  - stable completeness/legal-hold reason code
  - stable projection-applied reason code
  - typed fail-closed error for invalid input, owner-scope denial, lookup failure, or missing
    partition state

## Boundaries/Non-goals
- In scope:
  - extract the deterministic bookkeeping currently performed inside
    `project_partition_shred_completeness_with_port`
  - reuse the existing `DataLayerM10ComplianceProjectionPort` seam
  - reuse the extracted M10 partition registry state machine for partition mutation
  - keep `kamn-core` public projection methods behaviorally equivalent through wrapper mapping
  - add `kamn-data-layer` tests for happy path and fail-closed paths
  - update extraction docs/contracts for the new projection-bookkeeping slice
- Out of scope:
  - M8 registry adapter behavior
  - DID parsing/normalization rules
  - Phase-6 orchestration behavior
  - public API contract changes in `kamn-core`
  - adding dependencies

## Failure Modes
- empty `partition_message_ids` fails closed
- empty message id entries fail closed
- invalid `partition_month_id` fails closed
- owner-scope authorization denial fails closed
- projection port lookup failures fail closed
- projection port invalid-input failures fail closed
- mutation against a missing partition fails closed

## Acceptance Criteria
- [ ] deterministic M10 compliance-projection bookkeeping behind the existing port seam is owned by
      `kamn-data-layer`
- [ ] `kamn-core::DataLayerM10PartitionLifecycleRegistry::project_partition_shred_completeness_from_m8`
      and `project_partition_shred_completeness_with_port` keep current behavior and stable reason
      codes through compatibility delegation
- [ ] M8 adapter logic, DID normalization, and core error mapping remain in `kamn-core`
- [ ] `kamn-data-layer` tests cover happy path, legal-hold path, lookup failure, and invalid input
      handling on the extracted bookkeeping surface
- [ ] extraction docs/contracts record the new slice and CI remains green

## Files to touch
- `specs/6532-extract-m10-compliance-projection-bookkeeping.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m10_compliance_projection_bookkeeping.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_compliance_projection_bookkeeping_integration.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- `kamn-data-layer` should expose typed projection-bookkeeping errors that preserve whether failure
  came from invalid input, port authorization/lookup/input failure, or registry mutation failure.
- `kamn-core` must preserve the current public
  `DataLayerM10PartitionLifecycleError::{EmptyField,InvalidPartitionMonthId,OwnerScopeViolation,ComplianceProjectionFailed,PartitionNotFound}`
  behavior and stable reason-code strings at the compatibility boundary.
- No silent fallback or normalization is allowed.

## Test Plan
- Red:
  - add `kamn-data-layer` integration coverage for the extracted bookkeeping projector
  - update extraction docs contract markers for the new slice
- Green:
  - implement the bookkeeping projector in `kamn-data-layer`
  - delegate core projection methods through the extracted projector and existing adapter/error
    mapping
- Refactor:
  - remove duplicated bookkeeping helpers from `kamn-core`
  - keep adapter-only/core-only concerns in `kamn-core`
- Integration:
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates
  - run targeted `kamn-data-layer`, `kamn-core`, and `test_file_size_policy` coverage
