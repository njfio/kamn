# Plan — #4231

Status: Reviewed

## Approach

- Add `scripts/ci/check_admission_backpressure_ci_smoke_convergence.py` following established CI smoke checker structure.
- Add `scripts/ci/test_check_admission_backpressure_ci_smoke_convergence.sh` with deterministic pass/fail fixtures.
- Integrate checker test and required runtime smoke tests into `scripts/ci/test_ci_tools.sh`.

## Affected Areas

- `scripts/ci/check_admission_backpressure_ci_smoke_convergence.py`
- `scripts/ci/test_check_admission_backpressure_ci_smoke_convergence.sh`
- `scripts/ci/test_ci_tools.sh`

## Risks and Mitigations

- Risk: checker pattern drift from existing CI smoke governance checkers.
  - Mitigation: mirror reason-order normalization and fast-mode block parsing pattern used by existing checkers.
