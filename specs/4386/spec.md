# Spec — #4386 Task: Async API Lifecycle Controls with Deterministic Concurrency and Backpressure Enforcement

Status: Reviewed
Priority: P1
Parent: #4384
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Service API ingress governance must remain deterministic under lifecycle pressure. Backpressure and
lifecycle limit contracts need stable reason markers so CI and operations can classify failures
without ambiguity.

## Scope

In scope:
- Deterministic lifecycle/backpressure taxonomy markers in service API axum ingress live validation.
- Deterministic policy checker reason mapping and normalized reason output.
- Contract-lane marker propagation for lifecycle/backpressure governance.
- Ops/service contract docs parity updates.

Out of scope:
- API endpoint redesign.
- Global queueing architecture changes.

## Acceptance Criteria

AC-1: Lifecycle controls enforce deterministic configured limits in policy checks.

AC-2: Backpressure/lifecycle governance emits stable taxonomy markers and deterministic rejection reasons.

AC-3: Contract-lane output and policy JSON include normalized reason projection (`reason_codes_value`) and lifecycle taxonomy markers.

AC-4: Regression tests preserve deterministic behavior for limit-breach and unstable-outcome tamper cases.

AC-5: Docs reflect lifecycle/backpressure taxonomy and failure-mode contracts.

## Conformance Cases

- C-01 (AC-1, Functional): tampering `api_concurrency_limit_default` from expected value fails closed with `service_api_axum_policy_api_concurrency_limit_default_mismatch`.
- C-02 (AC-1, Functional): tampering `api_rate_limit_per_second_default` from expected value fails closed with `service_api_axum_policy_api_rate_limit_per_second_default_mismatch`.
- C-03 (AC-2, Regression): tampering lifecycle taxonomy version fails closed with deterministic lifecycle-taxonomy mismatch reason.
- C-04 (AC-2, Regression): removing lifecycle taxonomy CSV field fails closed with deterministic `service_api_axum_policy_required_field_missing:<field>` reason.
- C-05 (AC-3, Integration): policy/contract lane outputs include lifecycle taxonomy markers and `reason_codes_value=none` on GO paths.
- C-06 (AC-4, Regression): tampering backpressure projection status fails closed with deterministic marker-missing reason.
- C-07 (AC-5, Docs): service API and ops docs include lifecycle/backpressure taxonomy and validation command coverage.

## Success Metrics

- Repeated policy runs over identical reports emit identical `reason_codes_value` and lifecycle taxonomy marker values.
- No nondeterministic/reordered reason marker regressions in service API ingress contract suites.
