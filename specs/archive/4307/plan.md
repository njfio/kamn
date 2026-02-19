# Plan — #4307

Status: Reviewed

## Approach

- Add a new CI strategy section documenting composite checker commands, taxonomy, boundary markers, and fail-closed reasons.
- Add R27.29 closure section in production next-steps plan with chain and marker references.
- Extend docs-contract tests to assert new markers and fail closed on drift.

## Affected Areas

- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `scripts/ci/test_production_service_next_steps_contract.sh`

## Risks and Mitigations

- Risk: brittle docs-string checks.
  - Mitigation: assert marker semantics and command references, not narrative prose.

## Interfaces and Contracts

- New docs marker namespace: `transport_observability_tls_*` convergence markers.
