# Milestone Spec Index - R27.10 Durability, crash-recovery, and state-consistency hardening

Milestone: `R27.10 Durability, crash-recovery, and state-consistency hardening` (GitHub milestone #44)

## Intent

Define replay-consistency and durability-governance contracts that fail closed on cross-store drift, crash-recovery parity mismatches, and marker-policy divergence before release promotion.

## Scope

In scope:
- Cross-store replay consistency checker behavior and deterministic divergence taxonomy.
- CI durability checker and baseline drift policy contracts.
- Docs and runbook marker parity for replay-consistency and durability controls.

Out of scope:
- Historical forensic query interfaces.
- New long-running analytics pipelines outside bounded CI and local-heavy workflows.

## Issue Hierarchy

- Program epic: `#3812`
- Parent epic: `#4008`
- Story: `#4010`
- Tasks:
  - `#4013` - implement cross-store replay consistency checker with deterministic divergence taxonomy
  - `#4014` - enforce CI durability governance checker with baseline drift and docs parity contracts
- Subtasks:
  - `#4019`
  - `#4020`

## Validation Expectations

- Unit, functional, integration, and regression coverage for replay-consistency divergence detection.
- Deterministic taxonomy markers and reason-code lists suitable for policy verification.
- Docs contract tests fail closed on taxonomy marker drift.
