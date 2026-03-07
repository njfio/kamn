# 6540 Extract M11 Closure Evidence

## Objective
Extract the deterministic M11 closure-evidence acceptance policy from
`kamn-core` into `kamn-data-layer` while preserving the existing `kamn-core`
public API and behavior through compatibility re-exports.

## Inputs/Outputs
- Inputs:
  - `DataLayerM11ClosureEvidenceInput`
  - `DataLayerM11OperatorReadinessReport`
  - `DataLayerPrdCriticalScenarioConformanceReport`
  - performance/signoff boolean gate markers
- Outputs:
  - `DataLayerM11ClosureEvidenceReport`
  - `DataLayerM11ClosureAcceptanceDecision::{Accepted,Rejected}`
  - stable closure reason-code constants
  - typed fail-closed empty-release-marker errors

## Boundaries/Non-goals
- In scope:
  - move the deterministic M11 closure-evidence constants, input/output types,
    typed error, and evaluator into `crates/kamn-data-layer`
  - preserve `kamn-core` public imports and downstream behavior through a
    compatibility shim
  - add dedicated `kamn-data-layer` integration coverage for accepted,
    hardening-blocked, critical-scenario-blocked, evidence-gap-blocked, and
    empty-release-marker fail-closed paths
  - update extraction docs/contracts so M11 ownership records the closure
    evidence extraction slice
- Out of scope:
  - changes to closure-evidence decision semantics
  - changes to M11 hardening readiness semantics
  - changes to PRD critical-scenario conformance semantics
  - CI/workflow changes
  - adding dependencies

## Failure Modes
- empty or whitespace-only `release_marker` fails closed
- non-`Go` hardening readiness rejects closure
- non-conformant critical-scenario report rejects closure
- missing performance/security/chaos signoff evidence rejects closure
- combined blocking conditions must project deterministic reason-code ordering

## Acceptance Criteria
- [ ] `kamn-data-layer` exports the deterministic M11 closure-evidence surface
- [ ] `kamn-core` preserves current public API, typed errors, and stable
      reason-code constants through compatibility re-exports/shims
- [ ] dedicated `kamn-data-layer` tests cover accepted, hardening-blocked,
      critical-scenario-blocked, evidence-gap-blocked, combined-blocking, and
      empty-release-marker fail-closed paths
- [ ] existing `kamn-core` closure-evidence tests remain green without
      public-contract changes
- [ ] extraction docs/contracts record the new M11 closure-evidence slice and
      CI remains green

## Files to touch
- `specs/6540-extract-m11-closure-evidence.md`
- `crates/kamn-data-layer/src/lib.rs`
- `crates/kamn-data-layer/src/data_layer_m11_closure_evidence/mod.rs`
- `crates/kamn-data-layer/src/data_layer_m11_closure_evidence/types.rs`
- `crates/kamn-data-layer/src/data_layer_m11_closure_evidence/error.rs`
- `crates/kamn-data-layer/src/data_layer_m11_closure_evidence/evaluator.rs`
- `crates/kamn-data-layer/tests/data_layer_m11_closure_evidence_integration.rs`
- `crates/kamn-core/src/data_layer_m11_closure_evidence.rs`
- `crates/kamn-core/tests/data_layer_m0_m11_extraction_docs.rs`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/kamn-data-layer.md`
- `crates/kamn-data-layer/README.md`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Error Semantics
- `kamn-data-layer` must expose typed closure-evidence errors that preserve
  `DataLayerM11ClosureEvidenceError::EmptyReleaseMarker`.
- `kamn-core` compatibility exports must preserve the current public
  `DataLayerM11ClosureEvidence*` shapes and all stable reason-code constants.
- No silent fallback, normalization, or partial acceptance is allowed.

## Test Plan
- Red:
  - add a `kamn-data-layer` integration test file importing the extracted M11
    closure-evidence surface before it exists
  - update the extraction docs contract with required M11 closure-evidence
    markers before docs are updated
- Green:
  - implement the extracted M11 closure-evidence module in `kamn-data-layer`
  - replace the `kamn-core` file body with compatibility re-exports
- Refactor:
  - split the extracted ownership into a small module directory so the repo
    file-size constraint stays intact
  - remove duplicated closure-evidence logic from `kamn-core`
- Integration:
  - run `cargo fmt --all --check`
  - run strict clippy for touched crates
  - run targeted `kamn-data-layer`, `kamn-core`, extraction-docs, and
    test-file-inventory lanes

## Deviations
- None.
