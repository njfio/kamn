# Spec — #4387 Task: HTTP/WebSocket Protocol Checker Contracts and Deterministic Session-Failure Taxonomy

Status: Implemented
Priority: P1
Parent: #4384
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

WebSocket protocol/session validation evidence must fail closed with deterministic reason mapping so
promotion gates can audit protocol drift and invalid-session acceptance reliably.

## Scope

In scope:
- Deterministic policy checker reason mapping for websocket protocol/session reports.
- Normalized policy reason output marker (`reason_codes_value`).
- Docs-contract parity checks for websocket protocol/session taxonomy references.
- RED/GREEN websocket policy and contract-lane test coverage.

Out of scope:
- New websocket features/protocol extensions.
- Runtime semantic redesign of websocket routes.

## Acceptance Criteria

AC-1: Protocol/session violations are checker-detected and fail closed with deterministic reason codes.

AC-2: Policy output includes stable normalized reason projection (`reason_codes_value`) across pass/fail paths.

AC-3: Docs-contract parity checks fail closed when protocol/session taxonomy references drift from required docs.

AC-4: Unit/Functional/Integration/Regression coverage is present and passing.

## Conformance Cases

- C-01 (AC-1, Functional): tampered websocket lifecycle status marker fails with deterministic marker-missing reason.
- C-02 (AC-1, Regression): tampered websocket lifecycle taxonomy version fails with deterministic taxonomy mismatch reason.
- C-03 (AC-1, Regression): missing required taxonomy field fails with deterministic `service_api_websocket_policy_required_field_missing:<field>` reason.
- C-04 (AC-2, Functional): success path emits `reason_codes_value=none` in CLI and policy JSON.
- C-05 (AC-2, Regression): failure path emits deterministic `reason_codes_value` containing the active mismatch reason.
- C-06 (AC-3, Integration): live validation enforces release checklist websocket/protocol reason marker parity and fails when markers drift.
- C-07 (AC-4, Integration): websocket contract lane and policy suites pass with deterministic taxonomy output.

## Success Metrics

- Repeated policy runs over identical input emit identical `reason_codes_value` and taxonomy markers.
- No websocket protocol/session drift acceptance in policy and contract-lane tests.
