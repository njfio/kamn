# Milestone Index: R26.5

Milestone: R26.5 Observability and transport resilience hardening
GitHub Milestone: https://github.com/njfio/kamn/milestone/37
Status: In Progress

## Objective

Close remaining production-service observability and shared transport resilience gaps with deterministic
contracts, bounded CI cost, and local-heavy evidence lanes that stay excluded from fast-gate defaults.

## Scope

- Standardize runtime/service tracing taxonomy and deterministic observability reason markers.
- Harden observability route parity and secure-serving policy contracts.
- Enforce retry/backoff and reconnect policy determinism for shared transport clients.
- Keep CI-fast bounded while preserving local-heavy validation depth through explicit opt-in gating.

## Issue Hierarchy

- Epic: #3772
- Stories: #3773, #3774
- Tasks: #3775, #3776, #3777, #3778
- Subtasks: #3781, #3782, #3783, #3784, #3785, #3786, #3787, #3788

## Exit Signals

- Tracing and observability contracts emit deterministic schema and reason-code markers.
- Observability serving route parity fails closed on marker or policy drift.
- Shared transport retry/reconnect paths remain deterministic under transient failure.
- CI-fast excludes local-heavy run-mode lanes unless explicit workflow opt-in is supplied.
