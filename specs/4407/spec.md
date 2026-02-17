# Spec — #4407 Subtask: RED Tests for Fuzz Seed and Concurrency Misclassification Drift

Status: Reviewed
Priority: P1
Parent: #4402
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Current invariant-fuzz-concurrency policy tests do not explicitly fail on fuzz seed replay regression drift or concurrency race misclassification in reason mapping.

## Scope

In scope:
- Add failing tests for replay-count drift in fuzz/concurrency evidence.
- Add failing tests for concurrency lane fail misclassified as pass in reason payloads.

Out of scope:
- Policy checker implementation changes (handled in #4408).

## Acceptance Criteria

AC-1: Tests fail when fuzz replay evidence drifts from deterministic minimum coverage contract.

AC-2: Tests fail when concurrency race outcomes are misclassified in reason payload contracts.

AC-3: Existing deterministic pass/tamper coverage remains green.

## Conformance Cases

- C-01 (AC-1, Functional): fuzz replay test-count tamper fails closed.
- C-02 (AC-2, Functional): concurrency lane fail with mismatched reasons fails closed.
- C-03 (AC-3, Regression): existing taxonomy/artifact tamper paths remain deterministic.
