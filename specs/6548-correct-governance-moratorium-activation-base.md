# 6548 Correct Governance Moratorium Activation Base

## Objective
Move the governance/capability moratorium activation anchor to the merge commit that introduced the gate so the policy starts after activation instead of evaluating the activation PR itself.

## Inputs/Outputs
- Inputs:
  - `.ci/governance-feature-commit-ratio-moratorium.env`
  - `.github/workflows/ci-fast-gate.yml`
  - CI/docs contract tests covering the governance ratio gate markers
- Outputs:
  - Moratorium base SHA points at the activation merge commit `eded44be72ab5af7a709fd54809af745f918cb7a`
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
- [ ] `.ci/governance-feature-commit-ratio-moratorium.env` sets `GOVERNANCE_FEATURE_COMMIT_RATIO_MORATORIUM_BASE_SHA=eded44be72ab5af7a709fd54809af745f918cb7a`.
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
1. Update the workflow/docs contract assertions to expect the corrected merge-commit base SHA.
2. Run `bash scripts/ci/test_workflow_scope_policy.sh`.
3. Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_governance_feature_commit_ratio_gate_markers -- --exact --nocapture`.
4. Reproduce the gate locally on `main` by sourcing `.ci/governance-feature-commit-ratio-moratorium.env`, generating the subject list from `${GOVERNANCE_FEATURE_COMMIT_RATIO_MORATORIUM_BASE_SHA}..HEAD`, and running `python3 scripts/ci/check_governance_feature_commit_ratio.py`.
