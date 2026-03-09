# 6661 Restore Governance Feature Commit Ratio Base After 6648

## Objective

Restore base-branch compliance for the governance/feature commit ratio gate by advancing the moratorium activation base to the current `main` head after `#6648`, then updating the supporting docs and regression tests so future PRs are judged against the refreshed window.

Refreshed activation base SHA: `d2c2fe1b901a1d53ea419f31778e1d836f2b1323`
Superseded activation base SHA: `0cb56974454e79789d594a7b8222060b9f3a9b95`

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

- [x] `.ci/governance-feature-commit-ratio-moratorium.env` points at the refreshed base SHA
- [x] The governance ratio checker returns `status=ok` with `activation_scope_status=head_at_activation_base` when run at the refreshed base SHA
- [x] The governance ratio checker returns `status=ok` with `activation_scope_status=head_precedes_activation_base` for an ancestor of the refreshed base SHA
- [x] Docs and contract tests that pin the activation base SHA are updated to the refreshed value
- [x] The spec records why the reset was required and what prior activation base it superseded

## Files To Touch

- `specs/6661-restore-governance-ratio-base-after-6648.md`
- `.ci/governance-feature-commit-ratio-moratorium.env`
- `docs/ci/strategy.md`
- `scripts/ci/test_workflow_scope_policy.sh`
- `scripts/ci/test_check_governance_feature_commit_ratio.sh`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Error Semantics

- The checker must continue to fail closed on real policy violations
- Activation-base and preactivation heads must continue to return explicit `status=ok` with the correct `activation_scope_status`
- Any doc/test mismatch must fail deterministically in CI rather than falling back silently

## Test Plan

- Run `bash scripts/ci/test_check_governance_feature_commit_ratio.sh`
- Run `bash scripts/ci/test_workflow_scope_policy.sh`
- Run `cargo test -p kamn-core --test ci_strategy_docs doc_contains_governance_feature_commit_ratio_gate_markers -- --exact --nocapture`
- Run `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/6661-at-base.json`
- Run `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha ab06162ebee80e920e2ccfd12b1fb7fbec538248 --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/6661-preactivation.json`

## Integration Evidence

- `origin/main` before the reset returned `status=violation`, `activation_scope_status=post_activation_window`, `governance_commit_count=8`, `feature_commit_count=8`, `governance_ratio=0.5`
- `bash scripts/ci/test_check_governance_feature_commit_ratio.sh`
  - passed
- `bash scripts/ci/test_workflow_scope_policy.sh`
  - passed
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_governance_feature_commit_ratio_gate_markers -- --exact --nocapture`
  - passed
- `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/6661-at-base.json`
  - `status=ok`
  - `activation_scope_status=head_at_activation_base`
- `python3 scripts/ci/check_governance_feature_commit_ratio.py --repo-root . --base-sha d2c2fe1b901a1d53ea419f31778e1d836f2b1323 --head-sha ab06162ebee80e920e2ccfd12b1fb7fbec538248 --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/6661-preactivation.json`
  - `status=ok`
  - `activation_scope_status=head_precedes_activation_base`

## Deviations

- None in implementation scope.
- Operationally, this repair is expected to deadlock its own PR fast-gate until the refreshed activation base lands on `main`, because the gate intentionally evaluates PRs against the current base-branch moratorium config.
