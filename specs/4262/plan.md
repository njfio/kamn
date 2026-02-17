# Plan — #4262 Partition-Finality CI Smoke Docs Parity

Status: Reviewed

## Approach

1. Add strategy section describing checker command, reason taxonomy, budget markers, and exclusion policy.
2. Add production-plan closure section with active chain and deterministic closure markers.
3. Add docs-contract assertions in `crates/kamn-core/tests/ci_strategy_docs.rs`.

## Affected Surfaces

- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
