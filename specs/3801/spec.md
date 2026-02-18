# Spec - Issue #3801

- Title: Subtask: implement anti-flake merge-gate evaluator and reason taxonomy
- Parent: #3647
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Merge policy must deterministically block unresolved flaky conditions with explicit reason taxonomy markers.

## Objective

Close anti-flake merge-gate evaluator and reason-taxonomy coverage with deterministic policy tests.

## Scope

In scope:
- Anti-flake merge gate evaluator behavior.
- Anti-flake policy rule/taxonomy checks.
- Subtask closure artifacts.

Out of scope:
- CI platform architecture changes.

## Acceptance Criteria

- AC-1: Merge gate fails closed on unresolved flaky conditions.
- AC-2: Reason taxonomy/policy rules are deterministic and tested.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_anti_flake_merge_gate_policy.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_check_anti_flake_policy.sh` passes.
- C-03 (AC-3): both suites above pass in closure verification.

## Success Metrics

- Anti-flake merge policy deterministically blocks unresolved flaky conditions with stable reason markers.
