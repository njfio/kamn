# Issue #7145: Restore Governance Commit-Ratio Gate Coherence

## Objective

Make the governance/feature commit-ratio gate evaluate the commits introduced by the
current pull request instead of inheriting unrelated rolling history from the base
branch. Preserve the existing path classifier, 20 percent ceiling, report schema, and
fail-closed handling for unknown commits.

## Inputs And Outputs

### Inputs

- `github.event.pull_request.base.sha`
- `github.event.pull_request.head.sha`
- The checker and policy configuration loaded from the protected base branch.
- The PR's non-merge commit paths.

### Outputs

- A report classifying only commits in the PR base-to-head range.
- A passing result for a compliant one-spec/four-feature TDD history.
- A threshold violation for a one-spec/three-feature history.
- Deterministic tests that do not depend on the repository's moving `HEAD`.

## Boundaries And Non-Goals

- Do not change the 0.20 governance ceiling or 50-commit safety cap.
- Do not change governance path classification or unknown-commit failure behavior.
- Do not load executable checker code from the untrusted PR branch.
- Do not rewrite history or add synthetic commits to move a rolling window.
- Do not change MCP, Pi, receipt, escrow, settlement, or devnet behavior.
- Retain the moratorium base SHA for direct historical audits and compatibility tests;
  it no longer defines the PR gate's evaluated range.

## Failure Modes

- The workflow evaluates the fixed moratorium base through PR head and inherits
  base-branch debt.
- The workflow evaluates a merge commit instead of the explicit PR head SHA.
- The workflow loads checker code or thresholds from the PR branch.
- A PR with more than 20 percent governance-only commits passes.
- A commit with no classifiable path passes silently.
- Tests assert the moving repository `HEAD` and fail after unrelated merges.
- The legacy subject-file fallback evaluates a different range from path-based mode.

## Acceptance Criteria

- [ ] The workflow passes `github.event.pull_request.base.sha` as `--base-sha`.
- [ ] The workflow passes `github.event.pull_request.head.sha` as `--head-sha`.
- [ ] The legacy subject fallback uses the same PR base-to-head range.
- [ ] Checker code, helpers, and threshold configuration remain base-branch-owned.
- [ ] The 0.20 ceiling, 50-commit cap, path classifier, and unknown failure remain intact.
- [ ] Deterministic temporary-repository tests replace moving-`HEAD` compliance tests.
- [ ] One governance and four feature commits pass; one governance and three feature
  commits fail with `governance_commit_ratio_threshold_exceeded`.
- [ ] Workflow contract tests, checker tests, `make check`, and `make test` pass.
- [ ] The issue closure and PR summary report shell-surface DoD metrics.

## Files To Touch

- `specs/7145-pr-local-governance-ratio.md`
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/test_workflow_scope_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance.rs`
- Focused modules under
  `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance/`

## Error Semantics

- Above-threshold PR ranges return `status=violation` and
  `governance_commit_ratio_threshold_exceeded`.
- Empty or unclassifiable commits retain existing fail-closed reason codes.
- Missing base/head commits, invalid ancestry, missing base-branch payloads, or Git
  failures return a non-zero checker result with the existing error report shape.
- No fallback may silently widen the range to historical base-branch commits.

## Test Plan

### RED

- Require the workflow to use the pull request base SHA in both path and legacy subject
  modes.
- Require the workflow contract to reject the fixed moratorium SHA as the PR range base.

### GREEN

- Pass the explicit pull request base SHA to both checker interfaces.
- Update deterministic current-head contracts to exercise compliant and violating
  temporary PR ranges.
- Update CI strategy documentation for the PR-local boundary.

### REFACTOR

- Extract shared compliant/violating PR-range fixtures and remove moving-`HEAD`
  constants that no longer belong to the PR gate contract.
- Keep touched Rust and shell functions within repository size limits.

### INTEGRATION

- Exercise the workflow contract, Python checker fixtures, Rust range contracts, and
  full repository gates.
- Run the checker against this branch's actual merge-base-to-head range and record the
  report.

## Shell-Surface DoR

shell_loc_delta_estimate: 20
rust_loc_delta_estimate: 30
shell_to_rust_ratio_delta_estimate: 0.67
shell_surface_mitigation_issue: None
