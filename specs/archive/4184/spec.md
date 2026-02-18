# Spec — Issue #4184

- Title: add red tests for upgrade rehearsal lineage completeness and tamper rejection
- Parent: #4178
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Implemented
- Priority: P1

## Problem Statement

Upgrade rehearsal promotions depend on linked evidence across deployment preflight, live-node
validation, and go/no-go gate artifacts. Missing or tampered links must fail closed with
predictable diagnostics.

## Scope

In scope:
- add red fixtures for missing lineage links and tampered lineage markers,
- assert deterministic fail-closed reason outputs for lineage and promotion-gate mapping drift.

Out of scope:
- deployment architecture changes,
- artifact storage backend migration.

## Acceptance Criteria

- AC-1: red fixtures fail when linked artifact lineage is incomplete.
- AC-2: red fixtures fail when aggregate milestone lineage markers are tampered.
- AC-3: red fixtures fail when promotion gate mapping markers drift.
- AC-4: failures emit deterministic reason taxonomy and reason code ordering.

## Conformance Cases

- C-01: missing linked artifact fails with deterministic lineage reason. (AC-1)
- C-02: tampered milestone lineage payload fails closed with deterministic mismatch reason. (AC-2)
- C-03: promotion-gate final decision mismatch fails with deterministic mapping reason. (AC-3)
- C-04: checker output preserves deterministic taxonomy/version/csv markers. (AC-4)

## Success Metrics / Signals

- red fixtures execute in deploy go/no-go contract tests,
- each drift/tamper class maps to deterministic fail-closed reason markers.
