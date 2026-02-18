# Plan - Issue #3850

## Approach

1. Validate valid-path policy schema/final-decision behavior.
2. Validate deterministic fail-closed behavior for invalid/missing-reason scenarios.
3. Capture AC-to-test mapping in lifecycle artifacts.

## Affected Paths

- `specs/3850/spec.md`
- `specs/3850/plan.md`
- `specs/3850/tasks.md`

## Risks / Mitigations

- Risk: reason-taxonomy drift weakens policy determinism.
  Mitigation: preserve explicit reason-code assertions in conformance mapping.

## ADR

- Not required (lifecycle artifact closure only).
