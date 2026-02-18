# Spec — Issue #4156

- Title: add ci smoke checker for rehearsal-promotion marker lineage and local-heavy exclusions
- Parent: #4149
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md
- Status: Implemented
- Priority: P1

## Problem Statement

Rehearsal/promotion governance markers and boundary rules are documented, but there is no dedicated
CI smoke checker that enforces marker lineage and local-heavy exclusion contracts across workflow,
ci-tools, and closure docs.

## Scope

In scope:
- add a deterministic CI smoke checker for rehearsal/promotion marker-lineage and exclusion drift,
- add checker contract tests (pass + fail fixtures),
- wire docs/plan markers to the new checker contract.

Out of scope:
- running local-heavy rehearsal lanes in `ci-fast-gate`,
- release orchestration or deployment topology changes.

## Acceptance Criteria

- AC-1: checker fails closed when rehearsal/promotion CI smoke composition markers drift.
- AC-2: checker fails closed when local-heavy rehearsal commands leak into fast-gate surfaces.
- AC-3: checker enforces deterministic taxonomy/version and CI/local-heavy boundary markers in docs.
- AC-4: checker tests pass on baseline and emit deterministic reason codes on fixture drift.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | baseline workflow + ci-tools + docs | `status=pass`, `final_decision=GO`, convergence marker `verified` |
| C-02 | AC-1 | Regression | missing fast-mode rehearsal command fixture | fail with deterministic composition reason code |
| C-03 | AC-2 | Regression | leaked deep-lane rehearsal command in fast mode/workflow fixture | fail with deterministic exclusion reason code |
| C-04 | AC-3 | Functional/Regression | strategy/plan marker drift fixture | fail with deterministic docs marker reason code |
| C-05 | AC-4 | Regression | checker contract test lane | baseline pass + fixture failures validated |

## Success Metrics / Signals

- Rehearsal/promotion CI smoke governance has deterministic pass/fail signaling.
- Heavy local-only commands remain explicitly excluded from fast-gate.
- Plan and strategy marker drift is caught by fail-closed contract tests.
