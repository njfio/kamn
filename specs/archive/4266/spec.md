# Spec — #4266 Task: API Protocol Compliance Checker and Deterministic Mismatch Reason Mapping

Status: Implemented
Priority: P1
Parent: #4264
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

HTTP/API protocol compliance checks must emit deterministic, fail-closed mismatch semantics. Without stable mismatch reason mapping, promotions and runbook decisions can drift across runs.

## Scope

In scope:
- Deterministic protocol compliance checker outputs.
- Stable fail-closed mismatch reason mapping markers.
- Contract-lane integration checks and docs parity updates.

Out of scope:
- API feature redesign.
- Protocol framework replacement.

## Acceptance Criteria

AC-1: Checker validates required protocol markers deterministically.

AC-2: Mismatch conditions fail closed with stable reason outputs.

AC-3: Deterministic mismatch reason mapping markers are projected in checker/lane policy outputs.

AC-4: Unit/Functional/Integration/Regression/Conformance validation passes.

## Conformance Cases

- C-01 (AC-1, Functional): valid axum ingress protocol report yields `GO` with deterministic marker set.
- C-02 (AC-2, Regression): missing/tampered protocol markers reject with deterministic reason codes.
- C-03 (AC-3, Functional): mismatch reason mapping taxonomy/version/codes and resolved reason are emitted deterministically.
- C-04 (AC-3, Integration): axum ingress contract lane enforces mapping markers from policy checker output.
- C-05 (AC-4, Conformance): docs and docs-contract tests enforce mapping marker parity in ops/release documentation.

## Success Signals

- Repeated mismatch checks produce stable, ordered reason outputs.
- Mapping markers remain deterministic across policy checker and lane integration.
- Docs and docs-contract tests remain synchronized with checker contracts.
