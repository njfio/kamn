# Tasks — Issue #4156

- Status: Implemented

- [x] T1 (Red): add failing checker contract tests for missing composition markers and deep-lane leakage.
- [x] T2 (Green): implement rehearsal/promotion CI smoke convergence checker.
- [x] T3 (Green): wire checker tests into fast-mode CI tools and sync strategy/plan markers.
- [x] T4 (Regression): run targeted checker and docs contract tests.
- [x] T5 (Closeout): set spec lifecycle status to Implemented and post closure evidence.

## Completion Evidence

- `bash scripts/ci/test_check_rehearsal_promotion_ci_smoke_convergence.sh`
- `python3 scripts/ci/check_rehearsal_promotion_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120 --output-json /tmp/rehearsal-promotion-ci-smoke-convergence-report.json`
- `bash scripts/ci/test_production_service_next_steps_contract.sh`
