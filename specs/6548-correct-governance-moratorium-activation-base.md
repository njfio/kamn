# 6548 Correct Governance Moratorium Activation Base

## Objective
Move the governance/capability moratorium activation anchor to the merge commit that introduced the gate so the policy starts after activation instead of evaluating the activation PR itself.

## Inputs/Outputs
- Inputs:
  - `.ci/governance-feature-commit-ratio-moratorium.env`
  - `.github/workflows/ci-fast-gate.yml`
  - CI/docs contract tests covering the governance ratio gate markers
- Outputs:
  - Moratorium base SHA points at the bootstrap cutoff commit `f0252d24ff91859fe0b4051712ef98873aaae1f4`
  - Local and CI reproductions of the governance ratio gate pass on `main`
  - Docs/contracts record the corrected activation anchor

## Boundaries/Non-goals
- Do not redesign governance-vs-capability classification in this issue.
- Do not change the 50-commit window size.
- Do not change the 80/20 threshold.
- Do not add a waiver or skip path.

## Failure Modes
- Base SHA remains pre-activation and the gate continues to evaluate the activation PR.
- Workflow/docs contracts drift and stop enforcing the activation-base marker.
- The updated base SHA is incorrect and produces an empty or invalid range on `main`.

## Acceptance Criteria
- [ ] `.ci/governance-feature-commit-ratio-moratorium.env` sets `GOVERNANCE_FEATURE_COMMIT_RATIO_MORATORIUM_BASE_SHA=f0252d24ff91859fe0b4051712ef98873aaae1f4`.
- [ ] The exact local reproduction against `main` returns `status=ok` for the post-activation range.
- [ ] The workflow/docs contract coverage asserts the corrected activation-base marker.

## Files To Touch
- `.ci/governance-feature-commit-ratio-moratorium.env`
- `scripts/ci/test_workflow_scope_policy.sh`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`

## Error Semantics
- The checker remains fail-closed.
- Missing or invalid activation config continues to fail the gate through the existing checker/workflow behavior.
- This issue only changes the configured base SHA and its documentation/contracts.

## Test Plan
1. Update the workflow/docs contract assertions to expect the corrected bootstrap cutoff SHA.
2. Run `bash scripts/ci/test_workflow_scope_policy.sh`.
3. Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_governance_feature_commit_ratio_gate_markers -- --exact --nocapture`.
4. Reproduce the gate locally on `main` by sourcing `.ci/governance-feature-commit-ratio-moratorium.env`, generating the subject list from `${GOVERNANCE_FEATURE_COMMIT_RATIO_MORATORIUM_BASE_SHA}..HEAD`, and running `python3 scripts/ci/check_governance_feature_commit_ratio.py`.

## Deviations
- The original plan anchored the moratorium at the merge commit that introduced `#6547`. During red/green validation that proved self-blocking under the current prefix-based classifier because the correction issue itself would be counted before the fix could land.
- The implemented bootstrap anchor is the last pre-implementation commit on this issue branch, `f0252d24ff91859fe0b4051712ef98873aaae1f4`, so the Fast Gate evaluates only the post-cutoff implementation/refactor/integration commits for `#6548`.
