# 6542 Extract M7 Observability Projection

## Objective
Extract the deterministic M7 observability-sample projection from
`kamn-core` into `kamn-data-layer` while preserving the existing
`kamn-core::data_layer_m7_project_observability_sample(...) ->
ObservabilitySample` public contract through a compatibility wrapper.

## Inputs/Outputs
- Inputs:
  - `kamn-data-layer` owned M7 observability projection input struct populated
    from `kamn-core::DataLayerM7TelemetryPointRecord`
  - telemetry latency, activity, anomaly, active-session, and timestamp fields
    already stored on the core record
- Outputs:
  - `kamn-data-layer` M7 projection struct with observability sample fields
  - deterministic projection function for latency, throughput, error-rate, and
    availability values
  - unchanged `kamn-core::ObservabilitySample` return shape at the public
    wrapper boundary

## Boundaries/Non-goals
- In scope:
  - move the deterministic M7 observability projection math into
    `crates/kamn-data-layer`
  - introduce `kamn-data-layer` owned projection input/output types so the
    extraction does not depend on `kamn-core::DataLayerM7TelemetryPointRecord`
    or `kamn-core::ObservabilitySample`
  - preserve the existing `kamn-core` projector API by converting the extracted
    projection into `ObservabilitySample`
  - add dedicated `kamn-data-layer` coverage for latency, throughput,
    error-rate, and availability projection behavior
  - update extraction docs/contracts and the test-file inventory baseline
- Out of scope:
  - changes to `ObservabilitySample` semantics or field layout
  - changes to M7 telemetry ingest, rollups, billing, or owner-scope behavior
  - extraction of the wider `data_layer_m7_timeseries_telemetry` module
  - CI/workflow changes
  - adding dependencies

## Failure Modes
- None for the extracted projector surface.
- The extracted M7 projection function remains total and deterministic for any
  valid `DataLayerM7TelemetryPointRecord`.
- Existing `kamn-core` observability-monitor evaluation failures remain owned by
  the existing `ObservabilityMonitor` path and are not expanded by this slice.

## Acceptance Criteria
- [ ] `kamn-data-layer` exports a deterministic M7 observability projection
      surface owned by that crate, including the projection input/output types
- [ ] `kamn-core::data_layer_m7_project_observability_sample(...)` preserves its
      existing public signature and behavior through compatibility conversion
- [ ] dedicated `kamn-data-layer` tests cover latency mapping, throughput floor
      and session boost, error-rate zero/clamp behavior, and availability
      projection behavior
- [ ] existing `kamn-core` M7 telemetry tests remain green without public
      contract changes
- [ ] extraction docs/contracts record the new M7 slice and CI remains green

## Files to touch
- `specs/6542-extract-m7-observability-projection.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m7_observability_projection.rs`
- `crates/kamn-data-layer/tests/data_layer_m7_observability_projection_integration.rs`
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- The extracted `kamn-data-layer` M7 observability projector introduces no new
  typed error surface and must not silently rewrite or normalize output beyond
  the current deterministic clamping/floor rules.
- `kamn-core` must preserve the existing
  `DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE` constant and the
  existing `ObservabilityMonitor` error mapping path for downstream
  owner-observability evaluation.
- No silent fallback is allowed; this slice is projection-only and must preserve
  existing fail-closed behavior in the surrounding `kamn-core` evaluation path.

## Test Plan
- Red:
  - add a `kamn-data-layer` integration test importing the extracted M7
    observability projection surface before it exists
  - update the extraction docs contract with required M7 extraction markers
    before the docs are updated
- Green:
  - implement the extracted `kamn-data-layer` M7 projection module and export it
  - replace the `kamn-core` projector body with compatibility conversion from
    `DataLayerM7TelemetryPointRecord` into the extracted input type and then
    from the extracted output type into `ObservabilitySample`
- Refactor:
  - keep the new data-layer module and wrapper small enough to stay inside the
    repo file-size limits
  - remove duplicated M7 projection math from `kamn-core`
- Integration:
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates
  - run targeted `kamn-data-layer`, `kamn-core`, extraction-docs, and
    test-file-inventory lanes

## Deviations
- None.
