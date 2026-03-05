# Spec: Issue #6407 - M10 Phase-6 runtime clock validator extraction

## Objective

Extract deterministic Phase-6 scheduler runtime clock validation from `kamn-core` into `kamn-data-layer` policy contracts while preserving existing `kamn-core` behavior and error mapping.

## Inputs/Outputs

- Inputs:
  - `validate_phase6_scheduler_runtime_clock` in `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
  - policy evaluator module `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
  - extraction map markers in `docs/architecture/data-layer-runtime-wiring.md`
- Outputs:
  - exported data-layer runtime clock validator entrypoint
  - core helper delegation to extracted contract
  - docs/test marker updates proving extraction slice

## Boundaries/Non-goals

- In scope:
  - extracting runtime-clock validation only (`now_epoch_seconds > 0`, monotonic non-regression)
  - preserving current field/reason-code parity (`now_epoch_seconds` + scheduler signal invalid reason)
  - keeping public core APIs unchanged
- Out of scope:
  - scheduler runtime state machine extraction
  - trigger/budget policy semantics changes
  - API-breaking changes for downstream callers

## Failure modes

- FM-1: data-layer runtime clock validator not exported.
- FM-2: core wrapper maps invalid runtime clock fields incorrectly.
- FM-3: docs marker coverage missing.
- FM-4: integration evidence commands missing from spec.

## Acceptance criteria (testable booleans)

- [x] AC-1: `kamn-data-layer` exports runtime-clock validator entrypoint for Phase-6 scheduler policy.
- [x] AC-2: `kamn-core` runtime clock helper delegates to extracted contract with parity.
- [x] AC-3: core M10 contract lane remains green, including clock regression fail-closed tests.
- [x] AC-4: docs + data-layer policy contract tests include runtime-clock extraction markers/surface checks.
- [x] AC-5: Phase 6 integration evidence recorded for docs lane, data-layer policy lane, and core M10 lane.

## Files to touch

- `specs/6407-m10-phase6-runtime-clock-validator-extraction.md`
- `crates/kamn-data-layer/src/data_layer_m10_phase6_policy_evaluator.rs`
- `crates/kamn-data-layer/tests/data_layer_m10_phase6_policy_contract.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`

## Error semantics

- Data-layer validator must fail closed for invalid runtime clock inputs.
- Core wrapper must map extracted invalid-signal errors to existing `InvalidPhase6SchedulerSignal` with field `now_epoch_seconds` and existing reason code.
- No silent fallback behavior.

## Test plan

- RED:
  - add docs marker assertion for runtime-clock extraction slice.
  - extend data-layer policy contract test to call runtime-clock validator surface.
- GREEN:
  - implement exported data-layer runtime clock validator entrypoint.
  - delegate core runtime clock helper to extracted validator.
  - update docs markers.
- REFACTOR:
  - keep helper conversions and mapping DRY.
- INTEGRATION:
  - run docs extraction test.
  - run data-layer policy contract test.
  - run core M10 partition archival contract lane.
  - capture pre/post telemetry for core M10 lane.

## Phase 6 integration evidence

- 2026-03-05: `cargo test -p kamn-core --test data_layer_m0_m11_extraction_docs` (pass)
- 2026-03-05: `cargo test -p kamn-data-layer --test data_layer_m10_phase6_policy_contract` (pass)
- 2026-03-05: `cargo test -p kamn-core --test data_layer_m10_partition_archival` (pass)
- 2026-03-05 telemetry:
  - pre extraction lane timing: `m10_phase6_runtime_clock_pre_seconds=16.79`
    - command: `/usr/bin/time -f 'm10_phase6_runtime_clock_pre_seconds=%e' -o /tmp/m10_phase6_runtime_clock_pre.time cargo test -p kamn-core --test data_layer_m10_partition_archival --manifest-path /tmp/kamn-6407-pre/Cargo.toml`
  - post extraction lane timing: `m10_phase6_runtime_clock_post_seconds=0.11`
    - command: `/usr/bin/time -f 'm10_phase6_runtime_clock_post_seconds=%e' -o /tmp/m10_phase6_runtime_clock_post.time cargo test -p kamn-core --test data_layer_m10_partition_archival`

## Deviations

- None.
