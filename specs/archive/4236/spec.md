# Spec — #4236 Task: Journal Append Integrity Checks and Deterministic Checkpoint Marker Contracts

Status: Implemented
Priority: P1
Parent: #4234
Milestone: R27.25 Persistent Journal Replay and Checkpoint-Integrity Governance

## Problem Statement

Sqlite crash-recovery durability policy currently validates many markers, but append/checkpoint mismatch rejection coverage and explicit deterministic mapping for append-checkpoint integrity are incomplete.

## Scope

In scope:
- Add deterministic append-checkpoint integrity marker surface in crash-recovery checker outputs.
- Enforce fail-closed append/checkpoint parity mismatch policy checks.
- Add regression tests for WAL append-marker mismatch and parity mismatch.
- Update operations/release docs plus docs-contract tests for new markers.

Out of scope:
- Storage engine redesign.
- New persistence backend implementation.

## Acceptance Criteria

AC-1: Checker validates append/checkpoint integrity marker completeness and parity.

AC-2: WAL append/checkpoint mismatch outcomes fail closed with deterministic reasons.

AC-3: Red-to-green regression tests cover append mismatch and parity mismatch rejection.

AC-4: Ops + release docs include deterministic append/checkpoint governance markers.

## Conformance Cases

- C-01 (AC-1, Functional): dry-run crash-recovery lane emits append/checkpoint integrity markers with deterministic taxonomy.
- C-02 (AC-2, Regression): tampered `wal_append_status` fails with deterministic append mismatch reason.
- C-03 (AC-2, Regression): append/checkpoint parity mismatch fails with deterministic parity reason.
- C-04 (AC-3, Regression): contract-lane tests remain green with new append/checkpoint markers.
- C-05 (AC-4, Integration): docs-contract tests enforce ops/release marker parity.

## Success Signals

- Crash-recovery policy checker output includes deterministic append/checkpoint marker taxonomy.
- Tamper fixtures reject with stable reason codes.
- Docs/tests stay synchronized with checker output markers.
