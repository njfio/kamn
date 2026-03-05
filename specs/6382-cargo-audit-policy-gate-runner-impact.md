# Spec: Issue #6382 - Cargo-audit policy gate runner-impact measurement contract

## Objective

Close the remaining `#6382` gap by adding deterministic runner-impact measurement outputs and docs contracts for the existing cargo-audit CI policy gate.

## Inputs/Outputs

- Inputs:
  - cargo-audit policy checker: `scripts/ci/check_cargo_audit_policy.py`
  - policy checker contract tests: `scripts/ci/test_check_cargo_audit_policy.sh`
  - CI strategy documentation: `docs/ci/strategy.md`
  - docs contract suite: `crates/kamn-core/tests/ci_strategy_docs.rs`
- Outputs:
  - deterministic elapsed-time marker in cargo-audit policy output
  - policy contract tests covering elapsed-time marker/report field
  - documented runner-impact measurement and baseline markers in CI strategy docs
  - docs contract assertions for cargo-audit runner-impact markers

## Boundaries/Non-goals

- In scope:
  - measurement/reporting and docs parity for cargo-audit gate runtime impact.
  - no behavior regression in pass/fail policy decisions.
- Out of scope:
  - modifying `.github/workflows/**` CI definitions.
  - changing advisory severity threshold semantics.
  - changing waiver schema or approval workflow.

## Failure modes

- FM-1: policy output omits elapsed-time marker, leaving runner-impact untracked.
- FM-2: docs claim runner-impact measurement but marker names drift from implementation.
- FM-3: policy checker tests pass while elapsed-time fields are missing from JSON output.

## Acceptance criteria (testable booleans)

- [x] AC-1: cargo-audit policy checker emits deterministic elapsed-time metric marker and JSON field.
- [x] AC-2: policy checker contract tests validate elapsed-time marker presence and JSON field type.
- [x] AC-3: CI strategy docs include runner-impact measurement contract markers and baseline references for cargo-audit gates.
- [x] AC-4: docs contract tests enforce cargo-audit runner-impact markers.

## Files to touch

- `specs/6382-cargo-audit-policy-gate-runner-impact.md`
- `scripts/ci/check_cargo_audit_policy.py`
- `scripts/ci/test_check_cargo_audit_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Error semantics

- Existing fail-closed advisory/waiver policy semantics must remain unchanged.
- Elapsed-time measurement fields must never suppress existing violation outputs.
- Missing marker parity in docs must fail docs contract tests.

## Test plan

- RED:
  - extend policy checker contract test to require elapsed-time marker and JSON field; observe failure before implementation.
  - add docs contract assertions for cargo-audit runner-impact markers; observe failure before docs update.
- GREEN:
  - implement elapsed-time metric emission in checker output/report.
  - update docs markers and satisfy docs contract assertions.
- REFACTOR:
  - keep elapsed-time helper small and isolated from policy decision logic.
- INTEGRATION:
  - run cargo-audit checker contract test via dedicated script and CI tools suite.
  - run docs contract tests for CI strategy markers.

## Phase 6 integration evidence

- 2026-03-05: `bash scripts/ci/test_check_cargo_audit_policy.sh` (pass)
- 2026-03-05: `cargo test -p kamn-core --test ci_strategy_docs doc_contains_cargo_audit_runner_impact_measurement_markers -- --exact` (pass)
- 2026-03-05: `bash scripts/ci/test_ci_tools_command_surface_contract.sh` (pass)
- 2026-03-05: `bash scripts/ci/test_ci_strategy_contract.sh` (pass)

## Deviations

- None.
