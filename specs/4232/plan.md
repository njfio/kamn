# Plan — #4232

Status: Reviewed

## Approach

- Add admission/backpressure CI smoke governance section to `docs/ci/strategy.md`.
- Add R27.24 closure section to production next-steps plan with active chain and marker coverage.
- Extend docs-contract tests to enforce marker parity and drift rejection.

## Affected Areas

- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `scripts/ci/test_production_service_next_steps_contract.sh`

## Risks and Mitigations

- Risk: docs brittleness from string-based marker enforcement.
  - Mitigation: assert deterministic checker markers and command surfaces only.
