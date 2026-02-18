# Spec - Issue #3802

- Title: Subtask: remove stale flaky quarantine entries and enforce metadata policy
- Parent: #3646
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Stale flaky quarantine and ignored-test metadata can mask regressions and weaken merge governance.

## Objective

Close quarantine cleanup governance with deterministic flaky registry and metadata-policy checks.

## Scope

In scope:
- Flaky registry validation coverage.
- Ignored-test inventory drift and metadata-policy coverage.
- Subtask closure artifacts.

Out of scope:
- New quarantine model design.

## Acceptance Criteria

- AC-1: Flaky registry and ignored-test inventory checks detect stale state deterministically.
- AC-2: Metadata policy checks fail closed on malformed/stale entries.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_flaky_registry.sh` and `bash scripts/ci/test_check_ignored_test_inventory_drift.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh` passes.
- C-03 (AC-3): all suites above pass in closure verification.

## Success Metrics

- Quarantine and metadata governance checks remain deterministic and fail closed on drift.
