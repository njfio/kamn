# Spec — #4305 Subtask: TLS Reason Projection and Release-Gate Checker Integration

Status: Implemented
Priority: P1
Parent: #4298
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Release gating needs stable TLS failure reason projection so promotion decisions are deterministic and auditable.

## Scope

In scope:
- Deterministic reason projection for TLS evidence outcomes.
- Integration of projected reasons into go/no-go checker outputs consumed by release lanes.

Out of scope:
- Deploy orchestration redesign.

## Acceptance Criteria

AC-1: TLS reason projection is deterministic across repeated runs.

AC-2: Invalid TLS evidence fails closed with stable reason mapping in checker output.

AC-3: Integration tests validate checker output markers in end-to-end lane flow.

## Conformance Cases

- C-01 (AC-1, Unit/Functional): reason normalization emits stable ordering and serialization.
- C-02 (AC-2, Regression): stale/missing/malformed evidence cases map to deterministic reason codes.
- C-03 (AC-3, Integration): lane contract checks consume TLS taxonomy version and reason markers.

## Success Signals

- Repeated lane runs produce identical TLS reason marker values for identical input evidence.
