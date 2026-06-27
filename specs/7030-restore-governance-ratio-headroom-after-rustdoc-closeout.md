# 7030-restore-governance-ratio-headroom-after-rustdoc-closeout

## Objective
Restore PR #7022 Fast Gate governance/feature commit-ratio compliance after the
#7029 rustdoc closeout pushed the 50-commit window to 11 governance-only commits.

## Inputs/Outputs
- Inputs:
  - PR #7022 branch head `a320895b1278e976867b45c604bdef8a6076e9e0`
  - Fast Gate job `83788924592`
  - `.ci/governance-feature-commit-ratio-moratorium.env`
  - `scripts/ci/check_governance_feature_commit_ratio.py`
  - #7029 commits after `3a2f8c88`
- Outputs:
  - PR #7022 branch history removes separate #7029 evidence-only spec commits.
  - The final #7029 spec content remains present in the branch.
  - #7030 spec remains present as the issue trace for the history repair.
  - Governance/feature ratio passes locally and in PR Fast Gate.

## Boundaries/Non-goals
- Do not weaken Fast Gate thresholds, scripts, classifier behavior, or workflow
  logic.
- Do not add filler commits to push governance commits out of the 50-commit
  window.
- Do not change production/runtime behavior.
- Do not start MVP feature work before PR #7022 is green and merged.

## Failure Modes
- CI evaluates the pushed head after local evidence was collected on an earlier
  head.
- Evidence-only spec closeout commits occupy separate governance-only slots in
  the 50-commit window.
- A history rewrite drops #7029 evidence or loses the #7030 issue trace.

## Acceptance Criteria
- [x] Red evidence reproduces CI's governance-ratio failure at
      `a320895b1278e976867b45c604bdef8a6076e9e0`.
- [x] The repair preserves the final #7029 spec evidence.
- [x] The repair keeps one #7030 issue/spec trace in history.
- [x] The repair uses branch-only `--force-with-lease`, not a main push.
- [x] The governance/feature ratio gate passes locally at the repaired head with
      `governance_ratio <= 0.20`.
- [x] `cargo fmt --check`, strict workspace clippy, and `make check` remain green
      or are explicitly reused from an unchanged tracked tree.

## Files To Touch
- `specs/7030-restore-governance-ratio-headroom-after-rustdoc-closeout.md`
- Branch history for PR #7022

## Error Semantics
- Governance-ratio violations remain hard failures.
- Failed history rewrite, failed force-with-lease push, or missing evidence file
  is a hard blocker.
- No silent fallback to threshold edits or classifier changes.

## Test Plan
- Red: run `python3 scripts/ci/check_governance_feature_commit_ratio.py` against
  head `a320895b1278e976867b45c604bdef8a6076e9e0` and record the 0.22 failure.
- Green: fold #7029 evidence-only spec updates into an adjacent feature-path
  commit while preserving the final spec content.
- Verify: rerun the governance-ratio command at the repaired head.
- Regression: confirm the tracked tree content matches the already-validated
  #7029 tree except for the added #7030 spec, then rerun or reuse full local gates
  with explicit evidence.

## Completion Evidence
- Red: PR #7022 Fast Gate job `83788924592` failed at
  `a320895b1278e976867b45c604bdef8a6076e9e0` with
  `governance_commit_count=11`, `feature_commit_count=39`, and
  `governance_ratio=0.22`.
- Red: `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha a320895b1278e976867b45c604bdef8a6076e9e0 --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/kamn-governance-feature-commit-ratio-ci-repro-7030.json`
  reproduced the same failure locally.
- Green: #7029 evidence-only spec updates were folded into
  `refactor(7029): clarify bridge rustdoc link contract`, preserving the final
  #7029 spec content while removing two standalone governance-only commits.
- Green: `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha HEAD --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/kamn-governance-feature-commit-ratio-after-7030.json`
  passed with `governance_commit_count=10`, `feature_commit_count=40`, and
  `governance_ratio=0.2`.
- Refactor: no runtime/code simplification was applicable; the refactor was the
  branch-history fold that removed duplicate evidence-only closeout commits.
- Full gates: `cargo fmt --check` passed.
- Full gates: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.
- Full gates: `make check` passed.
- Regression: `git diff --check origin/7021-restore-local-quality-gates...HEAD`
  passed.
- Telemetry: `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json /tmp/kamn-shell-rust-ratio-after-7030.json`
  passed with `shell_to_rust_ratio=0.421487`.
- Telemetry: `bash scripts/ci/collect_shell_rust_loc_telemetry.sh --output-json /tmp/kamn-shell-rust-loc-telemetry-after-7030.json`
  passed with `delta_shell_line_total=229`, `delta_rust_line_total=166408`,
  and `delta_shell_to_rust_ratio=-0.181627` for the full PR branch.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: 0`
- `rust_loc_delta_estimate: 0`
- `shell_to_rust_ratio_delta_estimate: 0.0`
- `shell_surface_mitigation_issue: None`
- `shell_loc_delta_actual: 0`
- `rust_loc_delta_actual: 0`
- `shell_to_rust_ratio_delta_actual: 0.0`
- `shell_surface_ratio_target_status: neutral`
