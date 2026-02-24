# Spec: Issue #5885 - Shell LOC Reduction Wave 3 (Batched Capability Delivery)

- Issue: #5885
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The rolling non-merge history is governance-heavy, and shell-surface LOC remains above the desired trend. We need a batched capability wave that reduces shell LOC materially while preserving runtime/CI behavior and contract test outcomes.

## Scope
In scope:
- Whitespace-only compaction (redundant blank-line removal) in selected high-volume scripts under:
  - `scripts/ci/`
  - `scripts/runtime/`
  - `scripts/kolme/`
- Keep script behavior/outputs unchanged.
- Validate touched script contract lanes plus review-doc contract lane.

Out of scope:
- Any script logic change.
- Policy threshold or CI workflow contract changes.

## Acceptance Criteria
### AC-1 Shell LOC decreases
Given pre-change shell metrics,
When post-change metrics are collected,
Then `shell_line_total` is lower.

### AC-2 Touched script surfaces remain behaviorally equivalent
Given targeted script regression lanes,
When post-change scripts run,
Then each touched lane exits successfully.

### AC-3 Review docs contract remains green
Given review/docs contract tests,
When `cargo test -p kamn-core --test review_r53_docs_contract` runs,
Then it passes.

### AC-4 Issue is capability-batched to reduce structural coupling pressure
Given the issue branch history,
When implementation is complete,
Then the branch contains at least five non-spec/docs implementation commits under one lifecycle envelope.

## Conformance Cases
- C-01 (Integration, AC-1): `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json <path>` shows reduced `shell_line_total`.
- C-02 (Regression, AC-2): `scripts/ci/test_workflow_scope_policy.sh` passes.
- C-03 (Regression, AC-2): `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` passes.
- C-04 (Regression, AC-2): `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` passes.
- C-05 (Functional, AC-3): `cargo test -p kamn-core --test review_r53_docs_contract` passes.
- C-06 (Process, AC-4): `git log --no-merges --pretty=%s` on issue branch shows >=5 implementation commits.

## Success Metrics / Observable Signals
- `shell_line_total` decreases by at least 500 lines.
- All touched script regression lanes pass.
- Review docs contract lane remains green.
- Capability commit density improves by batching implementation commits in one issue.
