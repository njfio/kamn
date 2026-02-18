# Spec — Issue #4182

- Title: add red tests for compatibility mismatch taxonomy drift and runbook marker divergence
- Parent: #4177
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Implemented
- Priority: P1

## Problem Statement

Compatibility mismatch policy checks need explicit red fixtures proving taxonomy-marker drift and
runbook marker divergence fail closed before release approval.

## Scope

In scope:
- add failing fixtures for taxonomy marker drift and runbook divergence,
- assert deterministic fail-closed reason-code outputs for those fixtures.

Out of scope:
- checker redesign,
- runbook editorial restructuring.

## Acceptance Criteria

- AC-1: red fixtures fail when mismatch taxonomy markers drift.
- AC-2: red fixtures fail when runbook marker references diverge from checker markers.
- AC-3: regression coverage preserves deterministic taxonomy/runbook parity checks.

## Conformance Cases

- C-01: tampered mismatch taxonomy marker fixture fails closed with deterministic reason. (AC-1)
- C-02: missing runbook command marker fixture fails closed with deterministic reason. (AC-2)
- C-03: missing runbook taxonomy marker fixture fails closed with deterministic reason. (AC-2)
- C-04: contract lane preserves deterministic ordering and reason mapping under drift. (AC-3)

## Success Metrics / Signals

- red fixtures execute in contract-lane test harness,
- drift/divergence produces deterministic fail-closed reason markers.
