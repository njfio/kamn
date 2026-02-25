# Spec: Issue #5976 - Story: R57 High-Gap Non-Regression Guarding

- Issue: #5976
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: story
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5973

## Problem Statement
Previously high-risk R57 gaps appear remediated in current mainline, but non-regression controls need explicit strengthening and traceability.

## Scope
In scope:
- Strengthen integration checks for durable state + relay transitions.
- Strengthen workflow contract checks for live E2E wiring and required evidence outputs.
- Publish deterministic matrix mapping R57 high gaps to active checks.

Out of scope:
- New runtime features outside regression hardening.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Durable state + relay transition flow has deterministic integration guards.
- AC-2: Live E2E workflow contracts enforce required wiring and evidence markers.
- AC-3: R57 high-gap evidence matrix maps each gap to executable checks.

## Conformance Cases
- C-01 (Integration, AC-1): Created -> relayed -> delivered persisted flow verified across restart boundaries.
- C-02 (Conformance, AC-2): Workflow contract fails closed when required live-E2E markers/toggles are missing.
- C-03 (Functional, AC-3): Evidence matrix report includes complete high-gap mapping.

## Success Metrics / Observable Signals
- Deterministic pass/fail guard behavior for persistence/relay/live-E2E coverage.
- Gap-to-test mapping artifact produced and validated.
