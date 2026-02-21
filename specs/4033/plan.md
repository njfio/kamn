# Issue #4033 Plan - Deep-Scan Policy Checker and CI Dry-Run Governance

## Approach
1. Add deep-scan policy checker script under `scripts/runtime` and wrapper mapping in exec registry.
2. Add shell regression script for checker pass/fail + docs drift fixtures.
3. Add Rust checker contract tests covering unit/functional/integration/regression/performance.
4. Update `docs/ci/strategy.md` with deep-scan policy checker contract markers and CI dry-run governance markers.
5. Add `ci_strategy_docs.rs` parity assertions and wire checker shell test into `scripts/ci/test_ci_tools.sh` fast/full paths.

## Affected Modules
- `scripts/runtime/dependency_local_heavy_deep_scan_policy_contract.py`
- `scripts/runtime/check_dependency_local_heavy_deep_scan_policy.sh`
- `scripts/runtime/test_check_dependency_local_heavy_deep_scan_policy.sh`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/dependency_local_heavy_deep_scan_policy_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`
- `scripts/ci/test_ci_tools.sh`
- `specs/4033/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: runner/policy/docs marker drift.
  - Mitigation: checker validates runner constants and docs markers; docs tests assert marker parity.
- Risk: local-heavy run-mode command leakage into fast-gate.
  - Mitigation: fail-closed required/forbidden command-surface checks over ci-tools/workflow content.
- Risk: checker complexity/regression overhead.
  - Mitigation: bounded checker runtime and targeted shell/Rust tests.

## Interfaces / Contracts
- Policy report schema:
  `kamn.runtime.dependency-local-heavy-deep-scan-policy-report.v1`
- Policy reason taxonomy:
  `kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1`
- Policy reason codes:
  `dependency_local_heavy_deep_scan_policy_required_field_missing,dependency_local_heavy_deep_scan_policy_marker_mismatch,dependency_local_heavy_deep_scan_policy_reason_taxonomy_mismatch,dependency_local_heavy_deep_scan_policy_profile_contract_mismatch,dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch,dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_drift,dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_drift,ci_fast_gate_failed,dependency_local_heavy_deep_scan_policy_expected_decision_mismatch,dependency_local_heavy_deep_scan_policy_violation`

## Validation Strategy
- RED: add policy checker tests/docs assertions before checker/docs implementation.
- GREEN: implement checker/wrapper/docs/ci-tools wiring and rerun targeted suites.
- VERIFY: run fmt, clippy, targeted Rust tests, shell checker test, and ci-tools command-surface contract.
