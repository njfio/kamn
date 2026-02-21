# Issue #4003 Spec — Performance CI Smoke Docs-Contract Parity

- Status: Reviewed
- Issue: #4003
- Parent: #3997
- Milestone: R27.9 Throughput capacity and performance regression hardening

## Problem Statement
The performance CI smoke checker enforces threshold and selector/workflow contracts, but it does not fail closed when `docs/ci/strategy.md` marker blocks drift from checker contracts or when deterministic remediation markers are missing.

## Scope
In scope:
- Add docs marker parity validation for the Performance CI Smoke Threshold Governance Contract section.
- Add deterministic remediation marker validation keyed by performance smoke reason codes.
- Emit explicit docs status markers in checker output and JSON payload.
- Add/adjust shell + Rust tests and docs to cover new docs parity behavior.

Out of scope:
- Broader restructuring of CI strategy docs outside the performance smoke contract block.
- Changes to deep-lane performance behavior.

## Acceptance Criteria
- AC-1: Checker fails closed when required strategy-doc markers drift from performance smoke contract expectations.
- AC-2: Checker fails closed when any performance smoke reason code lacks a deterministic remediation marker in strategy docs.
- AC-3: Checker emits deterministic docs status markers (`performance_ci_smoke_docs_status`, `performance_ci_smoke_docs_remediation_status`) and reason codes in stable order.
- AC-4: Documentation and tests are updated so docs-parity/remediation regressions are covered by unit, functional, integration, and regression tests.

## Conformance Cases
- C-01 (Unit, AC-3): Baseline smoke checker run with canonical docs returns `status=pass`, docs statuses `verified`, and `performance_ci_smoke_reason_codes_value=none`.
- C-02 (Functional, AC-1): Missing required strategy-doc marker triggers fail with `performance_ci_smoke_docs_marker_parity_drift`.
- C-03 (Integration, AC-1): Checker invocation against alternate strategy-doc path with drifted marker block fails closed while threshold/selector/workflow inputs remain valid.
- C-04 (Regression, AC-2): Missing remediation entry for one reason code triggers fail with `performance_ci_smoke_docs_remediation_marker_missing`.
- C-05 (Conformance/Docs, AC-4): Strategy doc contains reason taxonomy, reason CSV, docs status expectations, remediation map version marker, and one remediation marker per reason code.

## Success Metrics
- Performance checker rejects docs/contract drift and missing remediation markers deterministically.
- CI strategy docs tests prevent future marker regressions.
- No regression in existing threshold/selector/workflow validations.

## AC → Tests Mapping
- AC-1: `scripts/ci/test_check_performance_thresholds.sh` (docs marker drift), `crates/kamn-core/tests/performance_ci_smoke_governance_contract.rs` integration test.
- AC-2: `scripts/ci/test_check_performance_thresholds.sh` (remediation marker missing), `crates/kamn-core/tests/performance_ci_smoke_governance_contract.rs` regression test.
- AC-3: `crates/kamn-core/tests/performance_ci_smoke_governance_contract.rs` unit baseline assertions.
- AC-4: `crates/kamn-core/tests/ci_strategy_docs.rs` docs parity/remediation coverage.
