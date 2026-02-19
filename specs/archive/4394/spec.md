# Spec — #4394 Subtask: RED Tests for Peer Integrity Drift and Retry-Timeout Misclassification

Status: Implemented
Priority: P1
Parent: #4388
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Current live transport fault-matrix tests do not fully lock peer adapter reason marker stability across sender-integrity drift and retry-timeout classification paths.

## Scope

In scope:
- RED assertions for deterministic peer reason markers in validation/policy outputs.
- RED tamper tests for peer reason mismatch detection.
- RED assertions for docs parity markers in contract-lane flow.

Out of scope:
- Runtime transport implementation redesign.

## Acceptance Criteria

AC-1: Tests fail when peer-integrity/retry-timeout reason markers are absent or drifted.

AC-2: Tests fail when tampered peer reason markers are not rejected deterministically.

AC-3: Tests fail when docs parity markers are not surfaced in contract-lane output.

## Conformance Cases

- C-01 (AC-1, Functional): validation output includes deterministic peer reason markers.
- C-02 (AC-1, Functional): policy output includes deterministic peer reason markers.
- C-03 (AC-2, Regression): tampered timeout reason marker fails with deterministic mismatch reason.
- C-04 (AC-2, Regression): tampered peer-integrity reason marker fails with deterministic mismatch reason.
- C-05 (AC-3, Integration): contract-lane output includes deterministic peer docs-parity marker projections.
