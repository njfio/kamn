# Spec — Issue #4157

- Title: update rehearsal-rollback governance docs and drift-contract tests for closure
- Parent: #4149
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md
- Status: Implemented
- Priority: P1

## Problem Statement

Rehearsal and rollback governance markers exist across CI and operator docs, but the
production closure plan is missing an explicit R27.19 closure section and contract assertions.
That gap allows taxonomy or boundary drift to go undetected during milestone closeout.

## Scope

In scope:
- add deterministic R27.19 closure markers to the production next-steps plan,
- add docs-contract regression coverage that fails closed on marker drift.

Out of scope:
- implementation of new rehearsal execution lanes,
- CI workflow topology changes.

## Acceptance Criteria

- AC-1: `docs/plans/2026-02-14-production-service-next-steps.md` includes R27.19 closure markers
  for rehearsal/rollback governance.
- AC-2: closure markers include low-cost CI smoke boundary and local-heavy opt-in boundary.
- AC-3: docs-contract tests fail closed when R27.19 markers drift or are removed.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | production next-steps plan doc | R27.19 closure section present with deterministic chain + marker taxonomy |
| C-02 | AC-2 | Functional | R27.19 closure section | `*_ci_smoke_max_seconds=120` and `*_local_heavy_max_seconds=900` markers present |
| C-03 | AC-3 | Regression | docs-contract test against plan doc | fails closed on missing R27.19 markers and passes on baseline |

## Success Metrics / Signals

- R27.19 closure evidence is explicitly documented in the production plan.
- Marker drift is caught by deterministic docs-contract tests in CI.
