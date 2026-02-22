# Issue #5588 Spec - PRD Phase-5c Spawn Timeline Contracts

- Status: Implemented
- Issue: #5588
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Process lifecycle state markers exist, but run output still lacks explicit ordered spawn timeline markers that encode deterministic orchestration sequencing before real process execution.

## Scope
In scope:
- Add deterministic `spawn_timeline` object to run output.
- Required ordered markers:
  - `postgres_start`
  - `kolme_start`
  - `kamn_nodes_start`
  - `agent_deploy_start`
- Canonical deterministic ordering values:
  - `step-1`, `step-2`, `step-3`, `step-4`
- Add RED->GREEN tests for marker presence and ordering.
- Add phase-5c docs marker artifact and milestone progression update.

Out of scope:
- Real process spawning.
- Live network execution.

## Acceptance Criteria
- AC-1: run output includes `spawn_timeline` object with required keys.
- AC-2: timeline ordering values are deterministic and canonical (`step-1..step-4`).
- AC-3: RED->GREEN tests validate timeline marker behavior.
- AC-4: phase-5c docs markers and milestone index are coherent.
- AC-5: quality gates pass (`fmt`, `clippy`, targeted + regression tests).

## Conformance Cases
- C-01 (AC-1): run output includes `spawn_timeline.postgres_start`.
- C-02 (AC-1): run output includes `spawn_timeline.kolme_start`.
- C-03 (AC-1): run output includes `spawn_timeline.kamn_nodes_start`.
- C-04 (AC-1): run output includes `spawn_timeline.agent_deploy_start`.
- C-05 (AC-2): all timeline values map to canonical `step-1..step-4` ordering.
- C-06 (AC-3): RED failures observed before implementation.
- C-07 (AC-3): GREEN passes observed after implementation.
- C-08 (AC-4): phase-5c docs marker artifact present.
- C-09 (AC-4): milestone index references #5588 as active issue.
- C-10 (AC-5): fmt/clippy/tests/regression commands pass.

## Success Metrics / Observable Signals
- `spawn_timeline` markers provide deterministic sequencing semantics.
- Timeline contracts are machine-readable and drift-protected by tests.
