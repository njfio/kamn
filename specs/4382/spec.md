# Spec — #4382 Subtask: composite gate taxonomy + CI smoke/local-heavy boundary enforcement

Status: Reviewed
Priority: P1
Parent: #4374
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement

Composite promotion gates require deterministic reason taxonomy outputs and strict CI smoke/local-heavy boundary markers.

## Scope

In scope:
- Implement composite reason taxonomy/version outputs.
- Implement boundary markers and fail-closed mismatch checks.
- Update docs and docs-contract assertions for boundary matrix.

Out of scope:
- Deep-lane workflow changes in fast CI path.

## Acceptance Criteria

AC-1: Composite gate reason taxonomy and reason-code CSV are deterministic.

AC-2: CI smoke/local-heavy boundary markers are deterministic and enforced.

AC-3: Missing/inconsistent composite evidence yields deterministic fail-closed reasons.

AC-4: Docs and docs tests reflect the composite boundary contract.

## Conformance Cases

- C-01 (AC-1, Functional): taxonomy/version outputs present in stdout/JSON.
- C-02 (AC-2, Functional): smoke/local-heavy boundary markers present and stable.
- C-03 (AC-3, Regression): mismatch/partial evidence fails closed with deterministic reasons.
- C-04 (AC-4, Integration): docs parity assertions pass.
