# Issue #3883 Plan

- Issue: #3883
- Status: Completed

## Approach
- Add a dedicated cutover CI exclusion policy checker that validates:
  - contract lane presence in `ci-fast-gate`
  - deep-lane exclusion from `ci-fast-gate` and `scripts/ci/test_ci_tools.sh`
  - strategy-doc marker and policy-command parity
- Add a contract test harness that exercises baseline GO behavior plus fail-closed drift scenarios.
- Wire the new contract test into aggregate CI tools execution.
- Add docs contract assertions in `kamn-core` to keep policy markers synchronized.

## Affected Modules
- scripts/cutover/
- scripts/ci/
- docs/ci/strategy.md
- crates/kamn-core/tests/ci_strategy_docs.rs

## Risks and Mitigations
- Risk level: low
- Mitigation: deterministic marker contracts plus drift/regression checks before rollout.

## Interface Contract
- No protocol or wire-format changes without explicit approval and ADR if needed.
- Runtime evidence outputs must remain deterministic and machine-checkable.
- Cutover policy checker emits deterministic marker taxonomy and fail-closed reason codes for CI-boundary drift.

## ADR
- No ADR required at planning stage; open ADR if dependency/protocol architecture changes emerge.
