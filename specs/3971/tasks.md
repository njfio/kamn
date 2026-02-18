# Tasks — Issue #3971

- [x] T1 (Red): add failing CI strategy contract markers for new wrapper-compatibility harness and fallback taxonomy strings.
- [x] T2 (Green): implement wrapper dispatch parity/legacy-entrypoint compatibility harness with deterministic marker checks.
- [x] T3 (Refactor): wire harness into `test_ci_tools.sh` and docs marker contract updates.
- [x] T4 (Verify): run targeted harness/strategy tests and fast CI tools regression.

## Planned Verification Commands

- `bash scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
