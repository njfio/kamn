# Spec — #4393 Subtask: Deterministic Protocol-Session Failure Taxonomy and Docs-Contract Parity Checks

Status: Implemented
Priority: P1
Parent: #4387
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

WebSocket protocol/session promotion evidence requires deterministic taxonomy projection and explicit
docs-contract parity checks against release governance markers.

## Scope

In scope:
- Deterministic websocket policy required-field mapping.
- Normalized reason marker output (`reason_codes_value`).
- Validation lane checks for protocol/session release checklist marker parity.
- Docs updates for protocol/session marker references.

Out of scope:
- Protocol semantics redesign.

## Acceptance Criteria

AC-1: Policy checker emits deterministic required-field and taxonomy mismatch reasons.

AC-2: Policy output includes deterministic `reason_codes_value` in JSON and CLI.

AC-3: Validation fails closed on protocol/session docs marker drift.

AC-4: Integration lane preserves taxonomy-to-gate flow with deterministic output markers.

## Conformance Cases

- C-01 (AC-1, Functional): missing required websocket taxonomy field maps to deterministic required-field reason.
- C-02 (AC-1, Regression): taxonomy version/csv mismatch map to deterministic mismatch reasons.
- C-03 (AC-2, Functional): success emits `reason_codes_value=none` in CLI + JSON.
- C-04 (AC-3, Integration): release checklist marker drift in validation lane fails closed with docs-contract reason.
- C-05 (AC-4, Integration): websocket contract lane output includes docs parity markers and stable policy reason projection.

## Success Metrics

- Stable deterministic websocket policy reason output across repeated runs.
- No acceptance of release-checklist protocol/session marker drift in websocket validation lane.
