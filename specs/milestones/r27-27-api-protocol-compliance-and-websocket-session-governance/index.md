# Milestone Spec Index — R27.27 API protocol compliance and websocket-session governance

Status: Active
Milestone: R27.27 API protocol compliance and websocket-session governance
Parent Program: #3812
Parent Epic: #4264

## Problem Statement

Promotion governance needs deterministic websocket-session protocol evidence and fail-closed CI smoke guardrails so marker drift is caught early without promoting heavy session drill cost into fast-gate.

## Scope

In scope:
- Websocket-session protocol marker drift detection in CI smoke scope.
- Deterministic fail-closed reason taxonomy for session marker mismatch and heavy-lane leakage.
- Docs + docs-contract synchronization for governance markers and boundary budgets.

Out of scope:
- Always-on heavy websocket session execution in fast-gate.
- CI topology redesign.

## Issue Specs Under This Milestone

- `specs/4269/spec.md` — websocket-session CI smoke governance task-level acceptance and closure.
- `specs/4276/spec.md` — checker implementation and exclusion enforcement.
- `specs/4277/spec.md` — docs and docs-contract synchronization.

## Verification Expectations

- Every AC maps to conformance case(s) and executable tests.
- CI smoke enforcement remains bounded and deterministic.
- Heavy websocket session drills stay excluded from fast-gate and CI tools fast mode.
