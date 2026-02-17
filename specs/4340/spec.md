# Spec — #4340 Story: Missing-Docs Graduation + Rustdoc/Architecture Navigation

Status: Reviewed
Priority: P1
Parent: #4338
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance

## Problem Statement

Documentation governance must stay auditable as module surfaces evolve. The story requires deterministic checks proving missing-docs graduation progress and stable rustdoc/navigation contracts.

## Scope

In scope:
- Task #4343 graduation governance hardening (with subtasks #4349 and #4350).
- Deterministic evidence emission for missing-docs allowlist/graduation deltas.

Out of scope:
- Full documentation corpus rewrite.
- Full-module fleet graduation in a single tranche.

## Acceptance Criteria

AC-1: Missing-docs graduation checks fail closed when expected graduation progress regresses or stagnates.

AC-2: Missing-docs checker emits deterministic allowlist/graduation delta evidence suitable for CI and reviewer audit.

AC-3: CI fast-gate and ci-tools contract lanes remain green with bounded runtime after governance updates.

AC-4: Story task chain closes with spec-backed tests and issue lifecycle updates.

## Conformance Cases

- C-01 (AC-1, Functional/Regression): synthetic allowlist drift and stagnation scenarios fail checker contract.
- C-02 (AC-2, Integration): missing-docs policy checker pass/fail output includes deterministic delta evidence markers.
- C-03 (AC-3, Integration/Performance): `scripts/ci/test_ci_tools.sh` fast mode remains green.
- C-04 (AC-4, Process): related task/subtask issues close with `status:done` and closure evidence comments.

## Success Signals

- No regressions in `scripts/ci/test_check_kamn_core_missing_docs_policy.sh`.
- Missing-docs checker output contains stable delta/governance markers.
- PR checks pass and merge cleanly.
