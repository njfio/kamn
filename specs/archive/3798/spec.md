# Spec - Issue #3798

- Title: Subtask: add CI-compatible flaky capture lane with deterministic summary markers
- Parent: #3645
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P1

## Problem Statement

CI-compatible flaky capture lanes require deterministic summary/report markers for auditable triage without fast-lane instability.

## Objective

Close capture-lane summary marker governance with deterministic report and registry-sync suites.

## Scope

In scope:
- CI-compatible quarantine-lane capture behavior.
- Deterministic flaky summary/report marker behavior.
- Subtask closure artifacts.

Out of scope:
- Enabling deep reproducer loops in fast-gate default path.

## Acceptance Criteria

- AC-1: CI-compatible capture behavior remains deterministic.
- AC-2: Summary/report markers are deterministic and auditable.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_run_cargo_test_with_quarantine.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_post_flaky_report_comment.sh` and `bash scripts/ci/test_sync_flaky_registry_issues.sh` pass.
- C-03 (AC-3): suites above pass in closure verification.

## Success Metrics

- CI-compatible flaky capture and summary markers remain deterministic and policy-auditable.
