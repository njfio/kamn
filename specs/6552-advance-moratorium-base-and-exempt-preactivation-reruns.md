# 6552 Advance Moratorium Base And Exempt Preactivation Reruns

## Objective
Advance the governance/feature commit-ratio moratorium start point past the rollout commits and make reruns of pre-activation PR heads pass fail-closed as historical, out-of-scope evaluations instead of tripping the post-activation ratio gate.

## Inputs/Outputs
- Inputs:
  - `.ci/governance-feature-commit-ratio-moratorium.env`
  - `scripts/ci/check_governance_feature_commit_ratio.py`
  - `scripts/ci/governance_feature_commit_ratio_support.py`
  - `scripts/ci/test_check_governance_feature_commit_ratio.sh`
  - `scripts/ci/test_workflow_scope_policy.sh`
  - `docs/ci/strategy.md`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
- Outputs:
  - base-branch CI evaluates post-rollout PRs from the new activation base
  - PR heads at or before the activation base return a non-violating historical result
  - strategy docs and contracts describe the activation semantics

## Boundaries/Non-goals
- Do not change the governance-versus-capability path taxonomy.
- Do not weaken the 80% capability target for post-activation commits.
- Do not broaden scope beyond the governance ratio checker, its config, docs, and regression tests.

## Failure modes
- Activation base remains before the rollout commits and rerunning `#6551` still reports `governance_commit_ratio_threshold_exceeded`.
- A PR head that predates activation still returns a ratio violation instead of an exempt/pass historical result.
- Post-activation governance-only history incorrectly passes because the historical exemption is too broad.
- Docs or workflow contracts drift from the new activation-base semantics.

## Acceptance criteria
- [ ] The checker returns a non-violating result when `head_sha` is equal to the configured activation base.
- [ ] The checker returns a non-violating historical result when `head_sha` is an ancestor of the configured activation base.
- [ ] The checker still returns `governance_commit_ratio_threshold_exceeded` for post-activation governance-only history.
- [ ] `.ci/governance-feature-commit-ratio-moratorium.env` anchors the moratorium after the rollout head used by `#6551`.
- [ ] `docs/ci/strategy.md` and `crates/kamn-core/tests/ci_strategy_docs.rs` document and pin the activation-base and pre-activation rerun semantics.

## Files to touch
- `.ci/governance-feature-commit-ratio-moratorium.env`
- `scripts/ci/check_governance_feature_commit_ratio.py`
- `scripts/ci/governance_feature_commit_ratio_support.py`
- `scripts/ci/test_check_governance_feature_commit_ratio.sh`
- `scripts/ci/test_workflow_scope_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Error semantics
- Invalid git arguments or repository state remain hard failures with `status=violation` and deterministic error payloads.
- Historical heads at or before activation are not errors; they must emit a non-violating report with explicit activation-scope markers.
- Post-activation ratio breaches remain hard failures with `reason_codes_csv=governance_commit_ratio_threshold_exceeded`.

## Test plan
- Add red regression coverage in `scripts/ci/test_check_governance_feature_commit_ratio.sh` for:
  - head equal to activation base
  - head ancestor of activation base
  - post-activation governance-only history still failing
- Add workflow/doc contract regressions in `scripts/ci/test_workflow_scope_policy.sh` and `crates/kamn-core/tests/ci_strategy_docs.rs` for the new base SHA and strategy markers.
- Re-run the shell regression tests and targeted docs contract after the implementation.
