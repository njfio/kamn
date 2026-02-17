# Spec — #4301 Subtask: Reconnect Sequencing with Bounded Backoff and Deterministic Reason Projection

Status: Implemented
Priority: P1
Parent: #4296
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Reconnect sequencing and backoff bounds need deterministic marker projection and policy enforcement so
retry envelope drift fails closed with auditable reasons.

## Scope

In scope:
- Deterministic reconnect sequence/backoff bound markers in retry diagnostics live lane outputs.
- Policy checker rules and reason codes for reconnect/backoff bound drift.
- Contract lane and docs updates for new markers/reasons.

Out of scope:
- Replacing runtime transport implementation.

## Acceptance Criteria

AC-1: Validation lane emits deterministic reconnect sequence/backoff envelope markers.

AC-2: Policy checker rejects reconnect/backoff envelope drift with stable reason codes.

AC-3: Contract lane/docs surfaces are synchronized with marker taxonomy and fail-closed reasons.

## Conformance Cases

- C-01 (AC-1, Functional): run/dry-run lane emits reconnect sequence/backoff envelope markers.
- C-02 (AC-2, Regression): reconnect bound drift rejected with deterministic reason code.
- C-03 (AC-2, Regression): backoff bound drift rejected with deterministic reason code.
- C-04 (AC-3, Conformance): contract lane output includes envelope marker and fail-closed marker parity.
- C-05 (AC-3, Conformance): ops configuration docs and docs tests include envelope governance markers.

## Success Signals

- Bounded reconnect/backoff envelope is deterministic and policy-enforced.
- Fail-closed reason projection is stable across retries and runs.
