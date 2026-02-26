# Spec: Issue #6048 - Add fail-closed governance/feature commit-ratio CI gate

- Issue: #6048
- Status: Implemented
- Type: task
- Priority: P1
- Area: governance
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6047

## Problem Statement
Release reviews currently compute governance/feature activity ratios as documentation markers only. Without a CI gate, PRs can continue to accumulate governance-heavy commit patterns (>50% governance ratio), reinforcing structural coupling and suppressing feature throughput.

## Scope
In scope:
- Add a deterministic checker that classifies non-merge PR commits into governance vs feature activity.
- Fail closed when governance ratio exceeds `0.50` or when commit classification is unknown/ambiguous.
- Emit schema-versioned JSON with counts, ratios, threshold, and reason codes.
- Wire checker into `ci-fast-gate` and add contract tests for script/workflow command surfaces.
- Update CI strategy docs with command contract and marker vocabulary.

Out of scope:
- Rewriting historical commits or backfilling old review documents.
- Organization-wide policy outside this repository.
- Changing the existing spec/review artifact taxonomy.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: PR fast-gate executes a governance/feature commit-ratio checker using non-merge commits from the PR range.
- AC-2: Checker fails closed when governance ratio exceeds `0.50`.
- AC-3: Checker fails closed when any commit subject cannot be classified deterministically.
- AC-4: Checker emits schema-versioned JSON report containing counts, ratios, threshold, and reason codes.
- AC-5: Contract tests fail when checker/workflow/doc command surfaces drift from the enforced policy.

## Conformance Cases
- C-01 (Conformance, AC-1): `ci-fast-gate` contains a dedicated governance-ratio step in the PR lane.
- C-02 (Functional, AC-2): fixture commit stream with governance ratio `<= 0.50` passes.
- C-03 (Regression, AC-2): fixture commit stream with governance ratio `> 0.50` fails with explicit reason code.
- C-04 (Regression, AC-3): fixture commit stream with unknown commit prefix fails closed with explicit reason code.
- C-05 (Conformance, AC-4): checker output JSON includes schema version, ratios, counts, threshold, and reason codes.
- C-06 (Integration, AC-5): command-surface and workflow-surface tests enforce wiring and arguments.

## Success Metrics / Observable Signals
- `scripts/ci/test_check_governance_feature_commit_ratio.sh` passes locally.
- `scripts/ci/test_ci_tools.sh` includes governance-ratio checker in fast lane.
- `scripts/ci/test_workflow_scope_policy.sh` validates governance-ratio workflow step and command.
- `.github/workflows/ci-fast-gate.yml` runs checker and uploads JSON artifact.
