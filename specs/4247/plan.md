# Plan — #4247

Status: Reviewed

## Approach

- Add a dedicated sqlite replay-integrity CI smoke governance section in `docs/ci/strategy.md`.
- Add an R27.25 closure section in production next-steps plan with active chain and marker references.
- Extend docs-contract tests in Rust/bash to enforce these markers.

## Affected Areas

- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `scripts/ci/test_production_service_next_steps_contract.sh`

## Risks and Mitigations

- Risk: brittle prose coupling.
  - Mitigation: pin only deterministic contract markers, command paths, and taxonomy strings.
