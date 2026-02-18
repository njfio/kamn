# R27.19 Live deployment rehearsal and rollback-governance hardening

## Milestone Summary

R27.19 closes governance gaps around deployment preflight completeness, rollback trigger
determinism, and rehearsal-to-promotion evidence lineage while keeping CI merge-gate checks
low-cost and fail-closed.

## Scope

In scope:
- deterministic preflight and rollback marker-policy contracts,
- rehearsal evidence lineage integrity and promotion reason-mapping parity,
- CI smoke governance for rehearsal/promotion drift with local-heavy lanes explicitly opt-in.

Out of scope:
- always-on heavy rehearsal execution in `ci-fast-gate`,
- deployment topology redesign or release process re-architecture.

## Active Chain

- Epic: `#4143`
- Story chain:
  - `#4144` deterministic deployment preflight and rollback trigger policy contracts
  - `#4145` live-node rehearsal evidence lineage and promotion-gate integrity
- Task chain:
  - `#4146` preflight marker completeness + fail-closed policy checks
  - `#4147` rollback trigger simulation + reason-taxonomy parity
  - `#4148` rehearsal evidence linked-artifact lineage verification
  - `#4149` low-cost CI smoke governance for rehearsal-promotion marker drift
- Subtasks:
  - `#4150`, `#4151` (preflight marker completeness and fail-closed output mapping)
  - `#4152`, `#4153` (rollback mismatch tests and parity implementation)
  - `#4154`, `#4155` (rehearsal lineage red tests and reason-code projection)
  - `#4156`, `#4157` (CI smoke checker + docs drift-contract closure sync)

## Governance Markers

- Rehearsal and promotion marker taxonomy remains deterministic across checker, CI, and docs.
- Local-heavy rehearsal lanes remain explicit opt-in and excluded from `ci-fast-gate`.
- Drift contracts fail closed when closure markers, budgets, or reason-taxonomy markers diverge.
