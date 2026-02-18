# Spec - Issue #3804

- Title: Subtask: add anti-flake merge evidence report and docs-contract guards
- Parent: #3647
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P1

## Problem Statement

Anti-flake merge policy must produce deterministic evidence and documentation parity checks for auditable governance.

## Objective

Close anti-flake evidence-report and docs-contract guard coverage for merge gate governance.

## Scope

In scope:
- Flaky report comment/evidence generation coverage.
- Registry sync and docs-policy parity coverage.
- Subtask closure artifacts.

Out of scope:
- New dashboard surfaces.

## Acceptance Criteria

- AC-1: Anti-flake evidence report markers are deterministic.
- AC-2: Docs/policy parity checks remain deterministic and fail closed on drift.
- AC-3: Conformance evidence is green and auditable.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_post_flaky_report_comment.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_sync_flaky_registry_issues.sh` and `bash scripts/ci/test_anti_flake_merge_gate_policy.sh` pass.
- C-03 (AC-3): suites above pass in closure verification.

## Success Metrics

- Anti-flake evidence reporting and docs-policy parity checks remain deterministic in CI.
