# 6549 Path-Based Capability Moratorium

## Objective
Harden the governance/capability moratorium so it classifies commits from changed surfaces instead of commit prefixes and so pull requests are evaluated with the moratorium policy files from the base branch rather than the PR head.

## Inputs/Outputs
- Inputs:
  - `scripts/ci/check_governance_feature_commit_ratio.py`
  - `scripts/ci/governance_feature_commit_ratio_support.py`
  - `.github/workflows/ci-fast-gate.yml`
  - `scripts/ci/test_check_governance_feature_commit_ratio.sh`
  - `scripts/ci/test_workflow_scope_policy.sh`
  - `docs/ci/strategy.md`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
- Outputs:
  - Governance-only commits are identified from governance-only changed paths.
  - Mixed or capability-surface commits count as capability work even if their commit prefix is `docs`, `chore`, or another governance-like prefix.
  - Fast Gate loads the moratorium checker/config from `origin/${base_ref}` so a PR cannot rewrite the policy that judges it.

## Boundaries/Non-goals
- Do not change the 50-commit window size.
- Do not change the 80/20 threshold.
- Do not broaden the moratorium beyond pull-request evaluation.
- Do not redesign `scripts/ci/select_targets.sh` in this issue.

## Failure Modes
- Governance-only commits still pass as capability because classification uses commit prefixes instead of changed paths.
- Capability commits are misclassified as governance because mixed-surface changes are not treated as capability work.
- The workflow still evaluates the PR head copy of the checker/config, letting policy changes grade themselves.
- The workflow becomes incompatible with older base-branch checker versions during rollout.

## Acceptance Criteria
- [ ] The checker supports classifying commits from repo path data between the moratorium base SHA and a target head SHA.
- [ ] Commits that change only governance surfaces classify as governance regardless of commit prefix.
- [ ] Commits that change any capability surface classify as capability regardless of commit prefix.
- [ ] The Fast Gate governance/capability step loads the checker/config from the base branch and remains compatible with the legacy checker interface during rollout.
- [ ] Targeted CI-tool tests cover governance-only, capability-only, mixed-surface, and base-branch compatibility cases.

## Files To Touch
- `scripts/ci/check_governance_feature_commit_ratio.py`
- `scripts/ci/governance_feature_commit_ratio_support.py`
- `scripts/ci/test_check_governance_feature_commit_ratio.sh`
- `scripts/ci/test_workflow_scope_policy.sh`
- `.github/workflows/ci-fast-gate.yml`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Error Semantics
- The checker remains fail-closed.
- Invalid git ranges, unreadable repo metadata, or unclassifiable commits return a violation report with deterministic reason codes.
- Base-branch policy loading failures in Fast Gate fail the job immediately.

## Test Plan
1. Add shell tests that create a temporary git repo with governance-only, capability-only, and mixed-surface commits and assert the reported counts.
2. Add a rollout-compatibility shell test that proves `ci-fast-gate.yml` can execute both the legacy `--commit-subjects-file` path and the new git-range path from base-branch policy files.
3. Run `bash scripts/ci/test_check_governance_feature_commit_ratio.sh`.
4. Run `bash scripts/ci/test_workflow_scope_policy.sh`.
5. Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_governance_feature_commit_ratio_gate_markers -- --exact --nocapture`.
