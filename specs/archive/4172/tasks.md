# Tasks — Issue #4172

- [x] T1 (Red): add/execute docs drift-contract assertions that fail before R27.20 markers exist.
  Evidence:
  - `bash scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh` drift fixtures fail
    closed when strategy/plan markers are removed.
- [x] T2 (Green): update strategy and production-plan docs with deterministic R27.20 marker
  lineage and boundary markers.
  Evidence:
  - Added custody/rotation convergence section to `docs/ci/strategy.md`.
  - Added R27.20 closure section to
    `docs/plans/2026-02-14-production-service-next-steps.md`.
- [x] T3 (Refactor): extend production next-steps docs contract required marker set for R27.20.
  Evidence:
  - `scripts/ci/test_production_service_next_steps_contract.sh` now asserts R27.20 closure markers.
- [x] T4 (Verify): run contract suites.
  Evidence:
  - `bash scripts/ci/test_production_service_next_steps_contract.sh`
  - `bash scripts/ci/test_check_custody_rotation_ci_smoke_convergence.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
