# 6538 Extract Shell Neutral Policy

## Objective
Extract the deterministic shell-neutral orchestration and ratio-budget policy
surface from `kamn-core` into `kamn-data-layer` while preserving the existing
`kamn-core` public API and behavior through compatibility re-exports.

## Inputs/Outputs
- Inputs:
  - `DataLayerShellNeutralPolicyInput`
  - `DataLayerPrdCriticalScenarioConformanceReport`
  - shell/rust LOC delta markers
  - current shell-to-rust ratio
  - warn/fail ratio thresholds
- Outputs:
  - `DataLayerShellNeutralPolicyReport`
  - `DataLayerShellNeutralPolicyDecision::{Verified,Warning,Blocked}`
  - typed shell-neutral reason codes and parse errors
  - typed fail-closed threshold validation errors

## Boundaries/Non-goals
- In scope:
  - move the deterministic shell-neutral constants, typed reasons, input/output
    types, parser, and evaluator into `crates/kamn-data-layer`
  - preserve `kamn-core` public imports and downstream behavior through a
    compatibility shim
  - add dedicated `kamn-data-layer` integration coverage for verified, warning,
    blocked, parse, and threshold-failure paths
  - update extraction docs/contracts to record the new shell-neutral tranche
- Out of scope:
  - changes to shell-neutral decision semantics
  - changes to PRD critical-scenario semantics
  - changes to M11 closure-evidence semantics
  - CI/workflow changes
  - adding dependencies

## Failure Modes
- non-finite threshold values fail closed
- zero or negative threshold values fail closed
- warn threshold greater than or equal to fail threshold fails closed
- unknown reason-code markers fail closed
- orchestration shell violations block policy
- positive shell LOC delta blocks policy
- ratio above fail threshold blocks policy

## Acceptance Criteria
- [ ] `kamn-data-layer` exports the deterministic shell-neutral policy surface
- [ ] `kamn-core` preserves current public API, typed errors, parser behavior,
      and reason-code markers through compatibility re-exports/shims
- [ ] dedicated `kamn-data-layer` tests cover verified, warning, orchestration
      blocked, positive-shell-delta blocked, ratio-fail blocked, threshold
      failure, and unknown-reason parse paths
- [ ] existing `kamn-core` shell-neutral policy tests remain green without
      public-contract changes
- [ ] extraction docs/contracts record the new tranche and CI remains green

## Files to touch
- `specs/6538-extract-shell-neutral-policy.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_shell_neutral_policy/mod.rs`
- `crates/kamn-data-layer/src/data_layer_shell_neutral_policy/types.rs`
- `crates/kamn-data-layer/src/data_layer_shell_neutral_policy/error.rs`
- `crates/kamn-data-layer/src/data_layer_shell_neutral_policy/evaluator.rs`
- `crates/kamn-data-layer/tests/data_layer_shell_neutral_policy_integration.rs`
- `crates/kamn-core/src/data_layer_shell_neutral_policy.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `docs/architecture/kamn-core-module-map.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- `kamn-data-layer` must expose typed shell-neutral errors that preserve
  `InvalidThresholdValue`, `InvalidThresholdOrder`, and
  `DataLayerShellNeutralPolicyReasonCodeParseError::UnknownReasonCode`.
- `kamn-core` compatibility exports must preserve the current public
  `DataLayerShellNeutralPolicy*` shapes and stable reason-code strings.
- No silent fallback, normalization, or partial acceptance is allowed.

## Test Plan
- Red:
  - add a `kamn-data-layer` integration test file importing the extracted
    shell-neutral surface before it exists
  - update the extraction docs contract with required shell-neutral markers
    before docs are updated
- Green:
  - implement the extracted shell-neutral module in `kamn-data-layer`
  - replace the `kamn-core` file body with compatibility re-exports
- Refactor:
  - split the extracted ownership into a small module directory so the repo
    file-size constraint stays intact
  - remove duplicated shell-neutral logic from `kamn-core`
- Integration:
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates
  - run targeted `kamn-data-layer`, `kamn-core`, extraction-docs, and
    test-file-inventory lanes
