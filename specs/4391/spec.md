# Spec — #4391 Subtask: Deterministic Async Rejection Reason Mapping for Lifecycle/Backpressure Violations

Status: Reviewed
Priority: P1
Parent: #4386
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Policy outcomes for async lifecycle/backpressure governance need deterministic reason mapping and
stable normalized output so repeated runs produce identical operational evidence.

## Scope

In scope:
- Policy checker deterministic mapping for lifecycle/backpressure taxonomy checks.
- Normalized `reason_codes_value` output in policy JSON and CLI output.
- Validation and contract-lane propagation of lifecycle taxonomy markers.

Out of scope:
- Runtime queue architecture redesign.

## Acceptance Criteria

AC-1: Rejection reason output is deterministic across repeated runs for identical report input.

AC-2: Policy checker validates lifecycle/backpressure taxonomy markers and emits deterministic mismatch reasons.

AC-3: Integration lane output includes lifecycle taxonomy markers and normalized reason output parity.

## Conformance Cases

- C-01 (AC-1, Functional): GO policy output contains `reason_codes_value=none` with stable ordering.
- C-02 (AC-1, Regression): tampered report outputs deterministic reason marker(s) in `reason_codes_value`.
- C-03 (AC-2, Functional): lifecycle taxonomy version mismatch triggers deterministic mismatch reason code.
- C-04 (AC-2, Regression): missing lifecycle taxonomy field triggers deterministic `service_api_axum_policy_required_field_missing:<field>` reason.
- C-05 (AC-3, Integration): contract-lane and policy JSON outputs expose lifecycle taxonomy markers with deterministic values.

## Success Metrics

- Deterministic reason projection is stable across policy runs in CI and local-heavy lanes.
- No contract-lane drift between summary markers and policy marker values for lifecycle taxonomy.
