# Issue #3937 Plan

- Issue: #3937
- Status: Implemented
- Spec: `specs/3937/spec.md`

## Delivery Approach
1. Implement checker-path enforcement in `#3942`:
   - remove cfg(test)-prefix false-negative behavior
   - preserve deterministic reason taxonomy and evidence outputs
2. Implement docs-contract parity in `#3943`:
   - add deterministic panic-policy remediation markers
   - enforce marker parity via `ci_strategy_docs` test
3. Validate using checker harness, docs-contract suite, and shell guardrails.

## Affected Modules
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: checker over/under-skips cfg(test) items.
  - Mitigation: RED fixture and fail-closed regression path in `#3942`.
- Risk: docs drift from checker contract/remediation guidance.
  - Mitigation: fail-closed docs-contract test added in `#3943`.

## Contracts and Interfaces
- Reason taxonomy contract: `kamn.ci.production-panic-replacement-reason-taxonomy.v1`
- Docs remediation markers contract:
  - `panic_path_policy_remediation_steps_version=v1`
  - `panic_path_policy_remediation_step_1..3`

## Verification Strategy
- RED/GREEN/REGRESSION evidence delivered in child PRs:
  - `#5156` (`#3942`)
  - `#5157` (`#3943`)
- Parent closeout verifies AC and conformance traceability.
