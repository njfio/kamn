# Spec: Issue #5979 - Task: Add R57 high-gap non-regression evidence matrix and guards

- Issue: #5979
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5976

## Problem Statement
R57 high-gap closures need deterministic mapping and guard coverage so resolved items cannot silently regress.

## Scope
In scope:
- Strengthen persistence + relay transition integration checks.
- Strengthen live-E2E workflow contract assertions.
- Add gap-to-test evidence mapping artifact/check.

Out of scope:
- New runtime feature additions.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Created -> relayed -> delivered persisted flow has deterministic regression guards.
- AC-2: Live-E2E workflow wiring/evidence contracts fail closed on drift.
- AC-3: Deterministic artifact maps each R57 high gap to active checks.

## Conformance Cases
- C-01 (Integration, AC-1): Relay projection and delivery transition regression tests pass and fail on drift fixtures.
- C-02 (Conformance, AC-2): Workflow contract tests enforce required jobs/env/evidence markers.
- C-03 (Functional, AC-3): Gap-to-check mapping artifact validates complete coverage.

## Success Metrics / Observable Signals
- Regression suite catches persistence/relay/live-E2E drift.
- Mapping artifact remains complete and test-validated.
