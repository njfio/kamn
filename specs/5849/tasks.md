# Tasks: Issue #5849

- [x] T1 (Tests first): Add `scripts/ci/test_check_e2e_live_workflow_contract.sh` with pass/fail fixtures and deterministic reason-code assertions.
- [x] T2 (Implementation): Add `scripts/ci/check_e2e_live_workflow_contract.py` enforcing live-marker + full SDK-direct scenario matrix invariants.
- [x] T3 (Implementation): Update `.github/workflows/e2e-live.yml` SDK-direct scenarios to `S-01..S-15`.
- [x] T4 (Integration): Wire checker tests into `scripts/ci/test_ci_tools.sh`.
- [x] T5 (Docs): Add CI strategy markers documenting the new live-workflow checker contract.
- [x] T6 (Verify): Run targeted checker tests, CI-tools fast mode, and shell syntax checks.
