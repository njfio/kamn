# Spec — #4241 Subtask: Deterministic Append-Checkpoint Checker Outputs and Fail-Closed Mapping

Status: Implemented
Priority: P1
Parent: #4236
Milestone: R27.25 Persistent Journal Replay and Checkpoint-Integrity Governance

## Problem Statement

Crash-recovery checker output lacks an explicit append-checkpoint integrity marker surface and parity-specific fail-closed mapping for release/runbook consumers.

## Scope

In scope:
- Add append-checkpoint integrity status/taxonomy markers to lane and policy output.
- Add deterministic append-checkpoint parity fail-closed reason mapping.
- Document markers in ops and release checklist docs.

Out of scope:
- New recovery flows.

## Acceptance Criteria

AC-1: Checker outputs deterministic append-checkpoint integrity markers.

AC-2: Append/checkpoint parity mismatch emits deterministic fail-closed reason.

AC-3: Release/ops docs reflect append-checkpoint marker taxonomy and reason mapping.

## Conformance Cases

- C-01 (AC-1): lane + policy outputs include append-checkpoint integrity marker fields.
- C-02 (AC-2): parity mismatch fixture fails with deterministic parity reason marker.
- C-03 (AC-3): docs-contract tests pass with new release/ops marker sections.
