# 6657 Restore Governance Feature Commit Ratio Base-Branch Compliance

## Objective

Restore base-branch compliance for the governance/feature commit ratio gate by advancing the moratorium activation base to the current compliant reset point on `main`, then updating the supporting docs and tests so future PRs are judged against the refreshed window.

Refreshed activation base SHA: `0cb56974454e79789d594a7b8222060b9f3a9b95`
Superseded activation base SHA: `e8a6de26ef277849b374e921c3e3307accbbacdf`

## Inputs/Outputs

- Inputs:
  - Current activation base in `.ci/governance-feature-commit-ratio-moratorium.env`
  - Current `origin/main` head SHA that should become the refreshed activation base
  - Existing governance ratio docs and contract tests
- Outputs:
  - Updated activation base file pointing at the refreshed SHA
  - Updated doc/test markers reflecting the new base
  - Passing coverage proving `head_at_activation_base` and `head_precedes_activation_base` remain accepted

## Boundaries/Non-goals

- Do not change the rolling window size or max governance ratio
- Do not weaken or bypass the governance/feature ratio gate
- Do not redesign commit classification rules
- Do not modify unrelated CI gates

## Failure Modes

- Activation base file still points at the stale SHA, so `main` remains in `post_activation_window` violation state
- Docs/tests continue asserting the superseded activation base SHA and drift from the real policy
- Regression coverage for `head_at_activation_base` or `head_precedes_activation_base` is lost, making future resets unsafe
- The recorded evidence does not explain why the reset was required, making the next policy rollover ambiguous

## Acceptance Criteria

- [ ] `.ci/governance-feature-commit-ratio-moratorium.env` points at the refreshed base SHA
- [ ] The governance ratio checker returns `status=ok` with `activation_scope_status=head_at_activation_base` when run at the refreshed base SHA
- [ ] The governance ratio checker returns `status=ok` with `activation_scope_status=head_precedes_activation_base` for ancestors of the refreshed base SHA
- [ ] Docs and contract tests that pin the activation base SHA are updated to the refreshed value
- [ ] The spec records why the reset was required and what prior window it superseded

## Files To Touch

- `.ci/governance-feature-commit-ratio-moratorium.env`
- `docs/ci/strategy.md`
- `scripts/ci/test_workflow_scope_policy.sh`
- `scripts/ci/test_check_governance_feature_commit_ratio.sh`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/6657-restore-governance-ratio-base-branch-compliance.md`

## Error Semantics

- The checker must continue to fail closed on real policy violations
- Activation-base and preactivation heads must continue to return explicit `status=ok` with the correct `activation_scope_status`
- Any doc/test mismatch must fail deterministically in CI rather than falling back silently

## Test Plan

- Run `bash scripts/ci/test_check_governance_feature_commit_ratio.sh`
- Run `bash scripts/ci/test_workflow_scope_policy.sh`
- Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_governance_feature_commit_ratio_gate_markers -- --exact --nocapture`
- Run `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha <refreshed_sha> --head-sha <refreshed_sha> --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/governance-feature-ratio-at-base.json`
- Run `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha <refreshed_sha> --head-sha <ancestor_of_refreshed_sha> --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/governance-feature-ratio-preactivation.json`

## Integration Evidence

- `origin/main` before the reset returned `status=violation`, `governance_commit_count=21`, `feature_commit_count=27`, `unknown_commit_count=2`, `governance_ratio=0.4375`
- `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha 0cb56974454e79789d594a7b8222060b9f3a9b95 --head-sha 0cb56974454e79789d594a7b8222060b9f3a9b95 --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/6657-at-base.json`
  - `status=ok`
  - `activation_scope_status=head_at_activation_base`
- `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha 0cb56974454e79789d594a7b8222060b9f3a9b95 --head-sha e8412d97bfd95519921cdb0f32b5d11d12fa27f2 --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/6657-preactivation.json`
  - `status=ok`
  - `activation_scope_status=head_precedes_activation_base`

## Deviations

- None in implementation scope.
- Operationally, this repair is expected to deadlock its own PR fast-gate until the new activation base lands on `main`, because the PR is judged against the stale base-branch policy by design.
