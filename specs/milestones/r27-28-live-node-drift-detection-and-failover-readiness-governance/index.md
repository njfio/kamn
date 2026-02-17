# Milestone Spec Index — R27.28 Live-node drift detection and failover-readiness governance

Status: Active
Milestone: R27.28 Live-node drift detection and failover-readiness governance
Parent Program: #3812
Parent Epic: #4278

## Problem Statement

Promotion evidence must fail closed when live-node drift marker contracts diverge, and failover-readiness evidence must remain deterministic and auditable across CI smoke and local-heavy lanes.

## Scope

In scope:
- Live-node drift marker contract validation and deterministic mismatch reasons.
- Failover-readiness evidence/taxonomy convergence checks.
- CI smoke guardrails with local-heavy deep-lane boundaries.

Out of scope:
- New failover architecture design.
- Fleet-wide orchestration redesign.

## Issue Specs Under This Milestone

- `specs/4281/spec.md` — live-node drift checker and deterministic mismatch reason mapping.
- `specs/4285/spec.md` — red tests for live-node drift marker mismatch rejection behavior.
- `specs/4286/spec.md` — deterministic drift checker outputs and fail-closed reason mapping.

## Verification Expectations

- Every AC maps to conformance case(s) and test(s).
- Fail-closed drift reason mapping remains deterministic.
- Docs and docs-contract tests stay synchronized with checker behavior.
