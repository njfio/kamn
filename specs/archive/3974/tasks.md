# Tasks — Issue #3974

- [x] T1 (Red): add failing first-batch fixture drift and allowlist-bypass regressions.
- [x] T2 (Green): enforce first-batch graduated module presence in missing-docs policy checker.
- [x] T3 (Refactor): document first-batch graduation contract behavior in CI strategy docs.
- [x] T4 (Verify): run targeted missing-docs/strategy contracts and fast CI tools regression.

## Planned Verification Commands

- `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
