# Plan — #4224

Status: Reviewed

## Approach

- Add a deterministic CI smoke convergence checker for service-api-axum admission/backpressure governance.
- Validate required smoke commands in `scripts/ci/test_ci_tools.sh` fast mode.
- Validate heavy run-command exclusion in `.github/workflows/ci-fast-gate.yml` and ci-tools fast mode.
- Update CI strategy and production next-steps docs with taxonomy and boundary markers.
- Extend docs-contract tests to pin marker presence.

## Affected Areas

- `scripts/ci/check_admission_backpressure_ci_smoke_convergence.py`
- `scripts/ci/test_check_admission_backpressure_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `scripts/ci/test_production_service_next_steps_contract.sh`

## Risks and Mitigations

- Risk: marker taxonomy drift across checker/docs/tests.
  - Mitigation: pin deterministic constants in checker and exact marker strings in docs-contract tests.
- Risk: runtime increase in fast mode.
  - Mitigation: enforce smoke-only test composition and a 120-second CI smoke budget marker.
