# Spec — Issue #4185

- Title: implement upgrade lineage verifier and deterministic promotion gate reason mapping
- Parent: #4178
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Implemented
- Priority: P1

## Problem Statement

Promotion decisions need a dedicated upgrade lineage verifier that deterministically evaluates
linked preflight/live/gate artifacts and emits stable reason mapping for fail-closed outcomes.

## Scope

In scope:
- implement an upgrade rehearsal lineage verifier checker,
- validate promotion gate reason mapping deterministically,
- integrate checker coverage into existing deploy contract tests,
- update docs and docs-contract tests for required lineage markers.

Out of scope:
- changes to external signing workflows,
- new CI workflow composition.

## Acceptance Criteria

- AC-1: lineage verifier emits deterministic pass/fail output markers.
- AC-2: missing/tampered links fail closed with deterministic reason codes.
- AC-3: promotion gate mapping drift fails closed with deterministic reason codes.
- AC-4: docs and docs-contract tests enforce checker command and taxonomy markers.

## Conformance Cases

- C-01: baseline aggregate evidence yields GO and empty reason set. (AC-1)
- C-02: missing/tampered lineage links yield deterministic NO-GO reasons. (AC-2)
- C-03: promotion gate mapping drift yields deterministic NO-GO reasons. (AC-3)
- C-04: release/planning docs include checker + taxonomy markers enforced by tests. (AC-4)

## Success Metrics / Signals

- verifier is executable and integrated with existing contract tests,
- deterministic reason taxonomy/version/csv outputs are asserted,
- docs tests fail closed when required markers drift.
