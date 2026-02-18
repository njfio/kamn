# Spec - Issue #3647

- Title: Task: enforce anti-flake merge gate policy with deterministic evidence
- Parent: #3631
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Merge governance must deterministically block unresolved flaky conditions with auditable reason markers and evidence output.

## Objective

Close anti-flake merge-gate policy enforcement by consolidating evaluator and evidence-report contract coverage.

## Scope

In scope:
- Merge-gate evaluator and reason-taxonomy checks (`#3801`).
- Evidence-report/docs-parity checks (`#3804`).
- Task closure lifecycle artifacts.

Out of scope:
- CI platform migration.

## Acceptance Criteria

- AC-1: Merge gate fails closed on unresolved flaky conditions.
- AC-2: Evidence output includes deterministic reason markers and remediation references.
- AC-3: Unit/Functional/Integration/Regression conformance remains green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_anti_flake_merge_gate_policy.sh` and `bash scripts/ci/test_check_anti_flake_policy.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_post_flaky_report_comment.sh` and `bash scripts/ci/test_sync_flaky_registry_issues.sh` pass.
- C-03 (AC-3): all suites above pass in closure verification.

## Success Metrics

- Anti-flake merge governance deterministically blocks unresolved flaky states with auditable evidence.
