# 6544 Extract M7 Billing Reconciliation Policy

## Objective
Extract the deterministic M7 owner-billing daily projection and statement
reconciliation policy from `kamn-core` into `kamn-data-layer` while preserving
`kamn-core::DataLayerM7TelemetryRegistry::{project_owner_billing_daily,
reconcile_owner_billing_daily}` public behavior through compatibility wrappers.

## Inputs/Outputs
- Inputs:
  - `kamn-data-layer` owned billing sample input rows populated from
    `kamn-core::DataLayerM7TelemetryPointRecord`
  - `kamn-data-layer` owned billing reconciliation input populated from
    `kamn-core::DataLayerM7BillingReconciliationInput`
  - owner-scoped daily bucket, message, byte, query, and embedding counts
- Outputs:
  - `kamn-data-layer` owned daily billing projection rows
  - `kamn-data-layer` owned billing reconciliation report
  - stable match/mismatch reason-code constants
  - unchanged `kamn-core` billing projection/reconciliation return shapes at
    the registry wrapper boundary

## Boundaries/Non-goals
- In scope:
  - move deterministic M7 owner-billing daily grouping and reconciliation math
    into `crates/kamn-data-layer`
  - introduce crate-owned input/output/error types so the extraction does not
    depend on `kamn-core` billing structs or telemetry record types
  - preserve the existing `kamn-core` registry methods by converting core types
    into the extracted input surface and mapping extracted outputs back into the
    current core return shapes
  - add dedicated `kamn-data-layer` coverage for grouped daily projection,
    match, mismatch, missing-projection zero totals, and invalid bucket
    fail-closed behavior
  - add dedicated core parity coverage without extending the already-large
    `data_layer_m7_timeseries_telemetry.rs` test file
  - update extraction docs/contracts and the test-file inventory baseline
- Out of scope:
  - changes to owner-scope authorization behavior
  - changes to telemetry ingest, hourly/daily agent rollups, network
    aggregates, or observability evaluation
  - extraction of the entire remaining `data_layer_m7_timeseries_telemetry`
    module
  - CI/workflow changes
  - adding dependencies

## Failure Modes
- reconciliation input with `bucket_day_epoch_seconds == 0` fails closed
- reconciliation input with a non-daily-aligned bucket fails closed
- missing projection rows for a requested day do not fail; projected totals stay
  at zero and the result deterministically becomes `Match` or `Mismatch`
  against those zero totals

## Acceptance Criteria
- [ ] `kamn-data-layer` exports a deterministic M7 billing
      projection/reconciliation surface owned by that crate, including its
      input/output/error types
- [ ] `kamn-core::DataLayerM7TelemetryRegistry::{project_owner_billing_daily,
      reconcile_owner_billing_daily}` preserves existing public behavior through
      compatibility conversion
- [ ] dedicated `kamn-data-layer` tests cover grouped daily projection, match,
      mismatch, missing-projection zero totals, and invalid bucket fail-closed
      behavior
- [ ] existing `kamn-core` M7 billing/reconciliation tests remain green without
      public-contract changes
- [ ] dedicated core parity coverage exercises the real registry path against
      the extracted policy without increasing the soft-warning file-size count
- [ ] extraction docs/contracts record the new M7 billing slice and CI remains
      green

## Files to touch
- `specs/6544-extract-m7-billing-reconciliation-policy.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m7_billing_reconciliation/mod.rs`
- `crates/kamn-data-layer/src/data_layer_m7_billing_reconciliation/types.rs`
- `crates/kamn-data-layer/src/data_layer_m7_billing_reconciliation/error.rs`
- `crates/kamn-data-layer/src/data_layer_m7_billing_reconciliation/policy.rs`
- `crates/kamn-data-layer/tests/data_layer_m7_billing_reconciliation_integration.rs`
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs`
- `crates/kamn-core/tests/data_layer_m7_billing_reconciliation_integration.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- `kamn-data-layer` must expose a typed billing reconciliation error preserving
  invalid daily bucket input as a fail-closed error.
- `kamn-core` must preserve
  `DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE`,
  `DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE`, and
  `DataLayerM7TimeseriesError::InvalidBucketDayEpochSeconds`.
- No silent fallback is allowed. Missing projected rows are an explicit
  deterministic zero-total path, not an error suppression path.

## Test Plan
- Red:
  - add a `kamn-data-layer` integration test importing the extracted M7 billing
    projection/reconciliation surface before it exists
  - add a dedicated core parity test importing the not-yet-existing extracted
    billing surface
  - update the extraction docs contract with required M7 billing extraction
    markers before docs are updated
- Green:
  - implement the extracted `kamn-data-layer` M7 billing module and export it
  - replace the core billing projection/reconciliation logic with compatibility
    conversion/wrapping around the extracted surface
- Refactor:
  - keep the extracted ownership split into small module files under the repo
    size limits
  - remove duplicated M7 billing math from `kamn-core`
- Integration:
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates
  - run targeted `kamn-data-layer`, `kamn-core`, extraction-docs, core/extracted
    parity, and test-file-inventory lanes

## Deviations
- None.
