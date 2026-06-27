# 7024-copy-governance-ratio-git-helper

## Objective
Repair the Fast Gate governance/feature commit-ratio step so the base-branch checker can import its git-range helper when run from a temporary directory.

## Inputs/Outputs
- Inputs:
  - `.github/workflows/ci-fast-gate.yml`
  - `scripts/ci/test_workflow_scope_policy.sh`
  - `crates/kamn-core/tests/fast_gate_governance_helper_contract.rs`
- Outputs:
  - Fast Gate copies `scripts/ci/governance_feature_commit_ratio_git.py` from `origin/${{ github.base_ref }}` into the same temp directory as the checker and support module.
  - Local contracts fail if the helper copy line is missing.
  - The governance ratio checker still uses the same base SHA, window size, threshold, and report path.

## Boundaries/Non-goals
- Do not change the governance ratio threshold.
- Do not change the moratorium activation base.
- Do not bypass, skip, or weaken the governance/feature ratio gate.
- Do not modify unrelated Fast Gate lanes.

## Failure Modes
- The workflow runs the base checker from a temp directory without copying all imported helper modules.
- The checker fails with `ModuleNotFoundError` before producing `ci-governance-feature-commit-ratio.json`.
- A future edit removes the helper copy line while keeping the git-range checker path.

## Acceptance Criteria
- [ ] `ci-fast-gate.yml` copies `scripts/ci/governance_feature_commit_ratio_git.py` into `$tmp_dir`.
- [ ] Local contracts fail before the workflow wiring fix.
- [ ] Local contracts pass after the workflow wiring fix.
- [ ] Exact base-checker temp-dir reproduction produces `status=ok` for this branch.
- [ ] Governance threshold, moratorium base, and report output semantics remain unchanged.

## Files To Touch
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/test_workflow_scope_policy.sh`
- `crates/kamn-core/tests/fast_gate_governance_helper_contract.rs`

## Error Semantics
- Missing helper wiring remains a hard Fast Gate failure.
- Ratio violations must still fail with the existing governance-ratio report semantics.
- This repair only makes the checker runnable from the temp dir; it does not change classification or thresholds.

## Test Plan
- Red: add contracts requiring the helper copy line and run them against the current workflow to confirm failure.
- Green: add the helper copy line in `ci-fast-gate.yml`, then rerun the targeted contracts.
- Integration: reproduce the CI temp-dir invocation using base-branch checker files and confirm `status=ok`.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +5`
- `rust_loc_delta_estimate: +50`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7024`
