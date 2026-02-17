# Spec — #4270 Subtask: RED Tests for API Protocol Marker Mismatch Rejection

Status: Implemented
Priority: P1
Parent: #4266
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Protocol marker mismatch behavior must fail closed deterministically; otherwise drift acceptance can appear non-reproducible and degrade governance confidence.

## Acceptance Criteria

AC-1: Tests fail on missing/tampered protocol marker fields.

AC-2: Repeated mismatch runs preserve deterministic reason-code ordering.

AC-3: Regression tests enforce deterministic mismatch mapping marker outputs.

## Conformance Cases

- C-01 (AC-1): tampered protocol marker status rejects with deterministic mismatch reason.
- C-02 (AC-1): tampered protocol taxonomy marker rejects with deterministic mismatch reason.
- C-03 (AC-2): repeated run with same tampered payload yields identical reason ordering.
- C-04 (AC-3): mismatch mapping marker outputs project deterministic expected values.
