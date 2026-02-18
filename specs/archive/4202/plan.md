# Plan — Issue #4202

## Approach

- Extend `scripts/ci/check_local_full_stack_integration_ci_smoke_convergence.py` to require
  `--plan-doc` and fail closed when production-plan markers drift.
- Extend `scripts/ci/test_check_local_full_stack_integration_ci_smoke_convergence.sh` with
  production-plan drift fixture and new checker argument wiring.
- Update governance docs for aligned marker taxonomy:
  - `docs/ci/strategy.md`
  - `docs/plans/2026-02-14-production-service-next-steps.md`
  - `docs/planning/kolme-devnet-ops.md`
- Extend docs-contract tests:
  - `scripts/ci/test_production_service_next_steps_contract.sh`
  - `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Affected Modules

- `scripts/ci/check_local_full_stack_integration_ci_smoke_convergence.py`
- `scripts/ci/test_check_local_full_stack_integration_ci_smoke_convergence.sh`
- `scripts/ci/test_production_service_next_steps_contract.sh`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `docs/planning/kolme-devnet-ops.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Risks and Mitigations

- Risk: checker and docs drift due duplicated marker text.
  - Mitigation: deterministic required-marker lists + docs contract tests.
- Risk: extra checker input could break existing invocations.
  - Mitigation: update all checker invocation points in strategy/tests/docs in same change.
- Risk: marker lists become stale after future refactors.
  - Mitigation: explicit fail-closed reason taxonomy and fixture-based tamper assertions.

## Interfaces/Contracts

- Checker CLI contract extends to include:
  - `--plan-doc <path>` (required)
- New deterministic reason code:
  - `production_plan_local_full_stack_convergence_markers_missing`
- Existing reason taxonomy list updated in docs/checker markers to include plan-drift reason.

## ADR

- Not required (docs + script contract synchronization, no dependency/protocol changes).
