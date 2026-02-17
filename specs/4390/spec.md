# Spec — #4390 Subtask: RED Tests for Async Lifecycle Limit Breaches and Unstable Backpressure Outcomes

Status: Reviewed
Priority: P1
Parent: #4386
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Current service API axum ingress policy tests do not sufficiently pin lifecycle limit-breach behavior
and deterministic backpressure reason projection under tamper scenarios.

## Scope

In scope:
- Add red tests for lifecycle limit-breach tamper scenarios.
- Add red tests for unstable/missing lifecycle taxonomy/backpressure projection markers.
- Add assertions for normalized deterministic reason output (`reason_codes_value`).

Out of scope:
- Policy checker implementation changes.

## Acceptance Criteria

AC-1: Tests fail when lifecycle limit defaults are tampered.

AC-2: Tests fail when lifecycle/backpressure taxonomy markers drift or are missing.

AC-3: Tests fail when normalized deterministic reason output is missing.

## Conformance Cases

- C-01 (AC-1, Functional): tampered `api_concurrency_limit_default` fails with deterministic mismatch reason code.
- C-02 (AC-1, Functional): tampered `api_rate_limit_per_second_default` fails with deterministic mismatch reason code.
- C-03 (AC-2, Regression): tampered lifecycle taxonomy version fails with deterministic mismatch reason code.
- C-04 (AC-2, Regression): missing lifecycle taxonomy CSV field fails with deterministic required-field reason code.
- C-05 (AC-2, Regression): tampered `async_lifecycle_backpressure_projection_status` fails with deterministic marker-missing reason code.
- C-06 (AC-3, Functional): policy output includes deterministic `reason_codes_value` markers for GO and NO-GO paths.
