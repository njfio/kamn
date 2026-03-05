# Spec: Issue #6403 - M10 Phase-6 scheduler preflight budget extraction

## Objective

Extract deterministic M10 Phase-6 scheduler preflight budget evaluation from `kamn-core` into `kamn-data-layer` policy contracts, while preserving existing `kamn-core` scheduler-cycle public APIs through compatibility wrappers.

## Inputs/Outputs

- Inputs:
  - preflight budget logic in `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
  - current policy evaluator contracts in `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
  - extraction markers in `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - new data-layer scheduler preflight budget evaluator contract entrypoint
  - core wrapper delegation/mapping preserving error and reason-code parity
  - docs/tests asserting the additional extraction marker surface

## Boundaries/Non-goals

- In scope:
  - extracting preflight budget count-shaping/evaluation into data-layer contracts
  - preserving scheduler-cycle deferred/applied and budget-preflight fail-closed behavior
  - keeping `kamn-core` scheduler-cycle signatures unchanged
- Out of scope:
  - full scheduler-cycle extraction from core
  - trigger precedence changes
  - API-breaking changes for downstream users

## Failure modes

- FM-1: preflight contract entrypoint missing from data-layer exports.
- FM-2: core wrapper maps invalid budget errors incorrectly.
- FM-3: scheduler preflight overflow reason codes regress.
- FM-4: docs marker/test coverage not updated.
- FM-5: integration evidence commands missing from spec.

## Acceptance criteria (testable booleans)

- [x] AC-1: `kamn-data-layer` exposes a scheduler preflight budget policy evaluator entrypoint.
- [x] AC-2: `kamn-core` scheduler-cycle preflight path delegates through extracted contract logic.
- [x] AC-3: existing M10 scheduler-cycle contract tests stay green, including preflight overflow assertions.
- [x] AC-4: docs + data-layer contract tests include preflight extraction markers/surface checks.
- [x] AC-5: Phase 6 integration evidence recorded for docs lane, data-layer policy lane, and core M10 lane.

## Files to touch

- `specs/6403-m10-phase6-scheduler-preflight-budget-extraction.md`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_policy_contract.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Data-layer preflight evaluator must fail closed with typed invalid-budget-field errors for zero/invalid budget limits.
- Core wrappers must map extracted evaluator errors into existing `DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget` with stable reason codes.
- No fallback behavior; preflight overflow and invalid-budget paths remain explicit failures.

## Test plan

- RED:
  - add docs marker assertion for scheduler preflight extraction slice.
  - add data-layer policy contract assertion for scheduler preflight evaluator entrypoint and exceeded path.
- GREEN:
  - implement data-layer preflight evaluator entrypoint.
  - replace core preflight evaluator logic with compatibility wrapper delegation.
  - update docs markers.
- REFACTOR:
  - reduce duplication in core conversion helpers where needed.
- INTEGRATION:
  - run docs extraction test.
  - run data-layer phase6 policy contract test.
  - run core M10 partition archival contract lane.
  - capture pre/post telemetry timing for core M10 lane.

## Phase 6 integration evidence

- 2026-03-05: `cargo test -p kamn-core --test data_layer_m0_m11_extraction_docs` (pass)
- 2026-03-05: `cargo test -p kamn-data-layer --test data_layer_m10_phase6_policy_contract` (pass)
- 2026-03-05: `cargo test -p kamn-core --test data_layer_m10_partition_archival` (pass)
- 2026-03-05 telemetry:
  - pre extraction lane timing: `m10_phase6_preflight_pre_seconds=16.67`
    - command: `/usr/bin/time -f 'm10_phase6_preflight_pre_seconds=%e' -o /tmp/m10_phase6_preflight_pre.time cargo test -p kamn-core --test data_layer_m10_partition_archival --manifest-path /tmp/kamn-6403-pre/Cargo.toml`
  - post extraction lane timing: `m10_phase6_preflight_post_seconds=0.11`
    - command: `/usr/bin/time -f 'm10_phase6_preflight_post_seconds=%e' -o /tmp/m10_phase6_preflight_post.time cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Deviations

- None.
