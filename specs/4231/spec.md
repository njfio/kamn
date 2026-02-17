# Spec — #4231 Subtask: Admission/Backpressure CI Smoke Checker

Status: Reviewed
Priority: P1
Parent: #4224
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement

Fast-gate needs a dedicated checker that fails closed on admission/backpressure marker drift and detects heavy service-api-axum run-command leakage.

## Scope

In scope:
- Implement checker and contract test for admission/backpressure CI smoke convergence.
- Emit deterministic reason taxonomy outputs and JSON schema markers.
- Enforce ci-fast-gate + ci-tools fast-mode exclusion for heavy run command.

Out of scope:
- Service API runtime contract behavior changes.
- New heavy lane execution in fast-gate.

## Acceptance Criteria

AC-1: Checker verifies required admission/backpressure smoke composition in fast mode.

AC-2: Checker fails closed on heavy run-command leakage in ci-tools fast mode and ci-fast-gate workflow.

AC-3: Checker emits deterministic reason taxonomy and stable marker output.

## Conformance Cases

- C-01 (AC-1, Functional): baseline checker returns GO with `reason_codes_value=none`.
- C-02 (AC-1, Regression): missing required smoke command fails with deterministic composition reason.
- C-03 (AC-2, Regression): leaked run command in fast mode fails with deterministic leakage reason.
- C-04 (AC-2, Regression): leaked run command in workflow fails with deterministic exclusion reason.
- C-05 (AC-3, Regression): max-seconds overflow fails with deterministic budget reason.
