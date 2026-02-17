# Spec — Issue #4183

- Title: implement mismatch taxonomy enforcement and runbook parity contract checks
- Parent: #4177
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Implemented
- Priority: P1

## Problem Statement

Deterministic compatibility governance requires checker outputs and runbook marker references to
remain synchronized; drift must fail closed in contract lanes.

## Scope

In scope:
- implement taxonomy/runbook parity enforcement in compatibility contract lane,
- add deterministic reason mapping for runbook parity violations,
- update runbook and docs-contract tests for required markers.

Out of scope:
- runbook workflow redesign,
- CI workflow expansion.

## Acceptance Criteria

- AC-1: contract lane enforces mismatch taxonomy marker parity deterministically.
- AC-2: runbook marker parity is validated and fails closed on drift.
- AC-3: regression checks capture taxonomy/runbook drift with deterministic reason codes.
- AC-4: docs and docs-contract tests include required parity markers.

## Conformance Cases

- C-01: baseline path retains GO decision with deterministic taxonomy markers. (AC-1)
- C-02: taxonomy drift triggers deterministic NO-GO reason mapping. (AC-1, AC-3)
- C-03: runbook command/taxonomy marker divergence triggers deterministic NO-GO reason mapping.
  (AC-2, AC-3)
- C-04: docs contracts fail closed when runbook parity markers are removed. (AC-4)

## Success Metrics / Signals

- contract lane rejects taxonomy/runbook parity drift deterministically,
- docs and docs-contract tests guard required parity markers.
