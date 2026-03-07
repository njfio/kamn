# 6536 Extract PRD Critical Scenario Conformance

## Objective
Extract the deterministic PRD critical-scenario conformance surface from
`kamn-core` into `kamn-data-layer` while preserving the existing `kamn-core`
public API and behavior through compatibility re-exports.

## Inputs/Outputs
- Inputs:
  - `DataLayerPrdCriticalScenarioResultInput`
  - scenario ids `62..71`
  - `DataLayerPrdCriticalScenarioMode::{RustOnly,ShellHybrid}`
- Outputs:
  - `DataLayerPrdCriticalScenarioResultRecord`
  - `DataLayerPrdCriticalScenarioConformanceReport`
  - `DataLayerPrdCriticalScenarioConformanceDecision::{Conformant,NonConformant}`
  - typed fail-closed conformance errors
  - stable reason-code constants for conformant, failed, missing, shell-policy,
    and invalid-mutation cases

## Boundaries/Non-goals
- In scope:
  - move the deterministic PRD conformance constants, types, matrix, and
    evaluator into `crates/kamn-data-layer`
  - preserve `kamn-core` public imports and downstream behavior through a
    compatibility shim
  - add dedicated `kamn-data-layer` integration coverage for required catalog,
    conformant evaluation, non-conformant evaluation, and fail-closed error
    paths
  - update extraction docs/contracts to record the new tranche
- Out of scope:
  - changes to shell-neutral policy behavior
  - changes to M11 closure-evidence behavior
  - public API redesign beyond implementation ownership moving to
    `kamn-data-layer`
  - adding dependencies

## Failure Modes
- empty `evidence_marker` fails closed
- invalid scenario ids outside `62..71` fail closed
- mutating an already-recorded result fails closed
- non-rust orchestration mode evaluates as non-conformant
- missing required scenario results evaluate as non-conformant
- failed required scenario results evaluate as non-conformant

## Acceptance Criteria
- [ ] `kamn-data-layer` exports the deterministic PRD critical-scenario
      conformance surface
- [ ] `kamn-core` preserves current public API, typed errors, and reason-code
      behavior through compatibility re-exports/shims
- [ ] dedicated `kamn-data-layer` tests cover deterministic required-scenario
      catalog, conformant evaluation, missing/failed/policy-violation
      non-conformance, and fail-closed invalid input and mutation paths
- [ ] existing `kamn-core` PRD conformance, shell-neutral policy, and M11
      closure-evidence tests remain green without public-contract changes
- [ ] extraction docs/contracts record the new tranche and CI remains green

## Files to touch
- `specs/6536-extract-prd-critical-scenario-conformance.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_prd_critical_scenario_conformance/mod.rs`
- `crates/kamn-data-layer/src/data_layer_prd_critical_scenario_conformance/types.rs`
- `crates/kamn-data-layer/src/data_layer_prd_critical_scenario_conformance/error.rs`
- `crates/kamn-data-layer/src/data_layer_prd_critical_scenario_conformance/helpers.rs`
- `crates/kamn-data-layer/src/data_layer_prd_critical_scenario_conformance/matrix.rs`
- `crates/kamn-data-layer/tests/data_layer_prd_critical_scenario_conformance_integration.rs`
- `crates/kamn-core/src/data_layer_prd_critical_scenario_conformance.rs`
- `crates/kamn-core/tests/data_layer_prd_critical_scenario_conformance.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `docs/architecture/kamn-core-module-map.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- `kamn-data-layer` must expose typed conformance errors that preserve
  `EmptyField`, `InvalidScenarioId`, and `InvalidResultMutation` behavior.
- `kamn-core` compatibility exports must preserve the current public
  `DataLayerPrdCriticalScenarioConformanceError` shape and all stable reason-code
  constants.
- No silent fallback, normalization, or partial acceptance is allowed.

## Test Plan
- Red:
  - add a `kamn-data-layer` integration test file importing the extracted PRD
    conformance surface before it exists
  - update the extraction docs contract with required PRD tranche markers before
    the docs are updated
- Green:
  - implement the extracted PRD conformance module in `kamn-data-layer`
  - replace the `kamn-core` file body with compatibility re-exports
- Refactor:
  - split the extracted ownership into a small module directory so the repo
    file-size constraint stays intact
  - remove duplicated conformance logic from `kamn-core`
- Integration:
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates
  - run targeted `kamn-data-layer`, `kamn-core`, extraction-docs, and
    test-file-inventory lanes

## Deviations
- `docs/architecture/kamn-core-module-map.md` was updated in addition to the
  original file list so the decomposition roadmap stopped listing
  `data_layer_prd_critical_scenario_conformance` as a remaining T4 surface
  after this extraction landed.
