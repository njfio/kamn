# Spec — #4303 Subtask: Implement Unified Observability Schema Checker and Deterministic Mismatch Reason Mapping

Status: Reviewed
Priority: P1
Parent: #4297
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Unified local-heavy observability policy currently cannot classify correlation schema drift and
cross-surface propagation mismatches with deterministic fail-closed reason codes.

## Scope

In scope:
- Implement deterministic correlation schema markers in unified run-lane report.
- Enforce correlation schema/propagation checks in unified policy checker.
- Add deterministic reason codes to unified policy taxonomy and contract-lane outputs.

Out of scope:
- New runtime observability collection backends.
- Changes to external wire protocols.

## Acceptance Criteria

AC-1: Checker validates required correlation schema markers deterministically.

AC-2: Checker validates API/runtime/Kolme correlation-id parity deterministically.

AC-3: Violations fail closed with stable reason taxonomy and deterministic reason ordering.

## Conformance Cases

- C-01 (AC-1, Functional): valid report with correlation schema markers passes policy.
- C-02 (AC-1, Regression): schema drift marker mismatch rejects with deterministic schema reason.
- C-03 (AC-2, Regression): API/runtime/Kolme correlation-id parity drift rejects with deterministic propagation reason.
- C-04 (AC-3, Conformance): contract-lane and policy outputs expose updated taxonomy/version marker set.

## Success Signals

- Existing and new unified local-heavy tests pass.
- Reason taxonomy remains deterministic between repeated runs.
