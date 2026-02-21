# Issue #5471 Spec - Live-Postgres Runtime Bundle Selector Integration

- Status: Implemented
- Issue: #5471
- Parent: #3812
- Milestone: R50.1 Live-postgres runtime bundle integration

## Problem Statement
Live-postgres multi-host execution bundle selector rows are defined in daemon test fixtures, but daemon runtime metadata currently exposes only a fixed row-count constant. This allows silent drift between runtime markers and the canonical selector set.

## Scope
In scope:
- Define live-postgres multi-host execution bundle selector rows in production runtime orchestration code.
- Derive daemon runtime row-count marker from selector-row source length.
- Add/extend tests that fail when selector rows and runtime row-count drift.

Out of scope:
- New live-postgres topology lanes.
- New CLI flags or report schema field additions.

## Acceptance Criteria
- AC-1: Runtime orchestration exposes a production selector-row source for the live-postgres multi-host execution bundle.
- AC-2: Daemon runtime row-count marker is computed from production selector-row source length (no disconnected magic number).
- AC-3: Contract tests fail on selector drift and pass when selector rows and runtime row-count are aligned.

## Conformance Cases
- C-01 (Functional, AC-1): Production selector-row source contains canonical multi-host execution bundle entries.
- C-02 (Unit, AC-2): Runtime row-count derivation equals selector-row source length.
- C-03 (Conformance, AC-3): Daemon runtime contract lane passes with selector/row-count alignment and would fail on mismatch.

## Success Metrics / Observable Signals
- Runtime metadata and selector-row source cannot diverge silently.
- Targeted daemon/runtime contract tests pass.
- No report/daemon regression in existing runtime contract tests.
