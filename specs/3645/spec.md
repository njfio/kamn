# Spec - Issue #3645

- Title: Task: build deterministic flaky-test reproducer and artifact capture pipeline
- Parent: #3631
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Reliable flaky triage requires deterministic reproducer execution and CI-compatible artifact capture/reporting behavior.

## Objective

Close flaky reproducer pipeline task by consolidating reproducer matrix and capture-lane summary contract coverage.

## Scope

In scope:
- Deterministic flaky reproducer matrix coverage (`#3800`).
- CI-compatible capture/summary/report coverage (`#3798`).
- Task lifecycle closure artifacts.

Out of scope:
- Root-cause fix implementation.

## Acceptance Criteria

- AC-1: Reproducer behavior is deterministic and auditable.
- AC-2: Capture/report behavior remains deterministic under CI-compatible constraints.
- AC-3: Closure evidence remains deterministic and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_run_flaky_reproducer.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_run_cargo_test_with_quarantine.sh`, `bash scripts/ci/test_post_flaky_report_comment.sh`, and `bash scripts/ci/test_sync_flaky_registry_issues.sh` pass.
- C-03 (AC-3): all suites above pass in closure verification.

## Success Metrics

- Flaky reproducer and capture/report lanes remain deterministic and policy-auditable.
