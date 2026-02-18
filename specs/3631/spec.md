# Spec - Issue #3631

- Title: Story: eliminate flaky main tests and harden merge gate reliability
- Parent: #3626
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Mainline CI credibility depends on deterministic flaky reproducer, root-cause stabilization, quarantine cleanup, and strict merge-gate policy.

## Objective

Close anti-flake reliability story by consolidating reproducer, root-cause stability, and merge-gate governance evidence.

## Scope

In scope:
- Reproducer and artifact capture pipeline (`#3645`).
- Root-cause stabilization and quarantine cleanup (`#3646`).
- Anti-flake merge-gate policy and evidence governance (`#3647`).
- Story lifecycle closure artifacts.

Out of scope:
- Broad CI platform migration.

## Acceptance Criteria

- AC-1: Flaky path has deterministic reproducer and stabilized behavior evidence.
- AC-2: Quarantine metadata and cleanup checks remain fail closed and auditable.
- AC-3: Merge-gate policy blocks unresolved flaky conditions with deterministic evidence markers.
- AC-4: Story-level conformance evidence remains deterministic and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_run_flaky_reproducer.sh`, `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh`, and `bash scripts/ci/test_run_cargo_test_with_quarantine.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_check_flaky_registry.sh`, `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`, and `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh` pass.
- C-03 (AC-3): `bash scripts/ci/test_anti_flake_merge_gate_policy.sh`, `bash scripts/ci/test_check_anti_flake_policy.sh`, `bash scripts/ci/test_post_flaky_report_comment.sh`, and `bash scripts/ci/test_sync_flaky_registry_issues.sh` pass.
- C-04 (AC-4): all suites above pass in closure verification.

## Success Metrics

- Anti-flake reproduction, stabilization, and merge-gate governance remain deterministic and policy-auditable.
