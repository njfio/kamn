# Plan — #4269

Status: Reviewed

## Approach

- Create RED fixtures for websocket-session marker drift and heavy-lane leakage failures.
- Implement dedicated CI smoke checker with deterministic reason taxonomy.
- Wire checker and websocket-session smoke tests into `scripts/ci/test_ci_tools.sh` fast/full paths.
- Update docs and docs-contract checks:
  - `docs/ci/strategy.md`
  - `docs/plans/2026-02-14-production-service-next-steps.md`
  - `scripts/ci/test_ci_strategy_contract.sh`
  - `scripts/ci/test_production_service_next_steps_contract.sh`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`

## Affected Areas

- `scripts/ci/*` checker and test harness scripts.
- `scripts/ci/test_ci_tools.sh` command-surface composition.
- `.github/workflows/ci-fast-gate.yml` (read-only validation target for exclusion checks).
- Strategy/plan docs and docs-contract tests.

## Risks and Mitigations

- Risk: docs/checker marker drift.
  - Mitigation: docs-contract assertions for required marker strings.
- Risk: heavy session drill command leaks into fast-gate.
  - Mitigation: explicit workflow and fast-mode leakage checks with deterministic reasons.

## Interfaces and Contracts

- Checker emits deterministic marker fields and reason taxonomy:
  - `websocket_session_ci_smoke_convergence_status`
  - `reason_taxonomy_version`
  - `reason_codes_csv`
- Fast-gate boundary remains smoke-only; heavy session drill remains opt-in/local-heavy.
