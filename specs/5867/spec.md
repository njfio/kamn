# Spec: Issue #5867 - End-to-End Message Delivery Continuity

- Issue: #5867
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Message send-to-recipient continuity requires explicit end-to-end lifecycle guarantees across send, relay projection, and recipient retrieval.

## Scope
In scope:
- Validate lifecycle continuity from send to recipient retrieval.
- Preserve authorization boundaries for non-recipient requesters.
- Add conformance/regression tests for lifecycle correctness.

Out of scope:
- Multi-node distributed delivery.

## Acceptance Criteria
- AC-1: Send creates durable lifecycle record + relay artifact.
- AC-2: Recipient retrieval transitions relay lifecycle deterministically.
- AC-3: Non-recipient retrieval never marks delivered.
- AC-4: Conformance/regression tests pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | send request | durable message + relay artifact |
| C-02 | AC-2 | Integration | recipient query | `delivered` transition |
| C-03 | AC-3 | Regression | non-recipient query | no invalid transition |
| C-04 | AC-4 | Verify | scoped tests | pass |
