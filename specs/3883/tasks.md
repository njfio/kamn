# Issue #3883 Tasks

- Issue: #3883
- Status: Completed

## Ordered Tasks
- T1 (Red): created drift/failure-path assertions in `scripts/cutover/test_check_cutover_ci_exclusion_policy.sh` for missing contract lane, deep-lane leakage, missing ci-tools coverage, and strategy docs marker drift.
- T2 (Green): implemented `scripts/cutover/check_cutover_ci_exclusion_policy.py` deterministic policy checker and JSON report output.
- T3 (Refactor): kept checker reason taxonomy/version marker centralized and aligned with strategy-doc marker contract strings.
- T4 (Regression): added policy checker harness to `scripts/ci/test_ci_tools.sh` and validated existing cutover rollback contract-lane tests still pass.
- T5 (Docs): updated `docs/ci/strategy.md` with dedicated cutover CI exclusion policy markers and command surface contract.
- T6 (Verify): added docs-contract assertions in `crates/kamn-core/tests/ci_strategy_docs.rs` and ran scoped script/cargo verification commands.

## Completion Evidence
- Passing commands:
  - `bash scripts/cutover/test_check_cutover_ci_exclusion_policy.sh`
  - `bash scripts/cutover/test_run_cutover_rollback_contract_lane.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_cutover_ci_exclusion_policy_contract_markers -- --exact`
- AC-to-test mapping is captured in `specs/3883/spec.md` conformance cases C-01..C-03.
