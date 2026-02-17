# Spec — #4343 Task: Missing-Docs Graduation Checker Evidence

Status: Reviewed
Priority: P1
Parent: #4340
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance

## Problem Statement

Graduation governance is enforced, but the top-level checker currently suppresses key velocity/allowlist evidence. Review and CI need deterministic emitted deltas to audit graduation progress.

## Scope

In scope:
- Red/green tests for graduation regressions and stagnation signaling (#4349).
- Deterministic allowlist/graduation delta evidence outputs in missing-docs checker (#4350).

Out of scope:
- New module graduation rollout beyond policy/evidence plumbing.

## Acceptance Criteria

AC-1: Checker tests fail for missing graduation progress and allowlist stagnation/regression conditions.

AC-2: `check_kamn_core_missing_docs_policy.sh` emits deterministic allowlist/graduation evidence markers on both pass and velocity-policy failure paths.

AC-3: Existing missing-docs policy behavior remains compatible (no loss of prior drift guards).

AC-4: Fast-gate CI tools contract lane remains green.

## Conformance Cases

- C-01 (AC-1, Regression): stagnation fixture mutation fails with velocity reason markers.
- C-02 (AC-2, Integration): checker pass output includes deterministic evidence fields (counts/deltas/reason markers).
- C-03 (AC-2, Integration): checker fail output for stagnation includes deterministic evidence fields.
- C-04 (AC-3, Functional): prior README/plan/architecture/rustdoc drift checks still fail as expected.
- C-05 (AC-4, Performance/Integration): CI tools fast mode remains pass.
