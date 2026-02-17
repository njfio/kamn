# Spec — #4296 Task: Deterministic Retry/Backoff/Reconnect Envelopes for Live Kolme Transport Flows

Status: Reviewed
Priority: P1
Parent: #4294
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Live retry/reconnect behavior must stay deterministic, bounded, and auditable. Drift in retry
exhaustion behavior or reconnect envelope limits can silently weaken fail-closed guarantees.

## Scope

In scope:
- Deterministic retry-envelope marker emission for local retry diagnostics live lane.
- Bounded reconnect sequencing/backoff envelope enforcement with stable policy reasons.
- Fail-closed reason projection on envelope exhaustion and envelope-bound drift.
- Docs updates for retry/reconnect envelope governance.

Out of scope:
- Transport protocol redesign.
- New runtime transport stack implementations.

## Acceptance Criteria

AC-1: Retry/backoff/reconnect envelope markers are deterministic and policy-checked.

AC-2: Reconnect attempt sequencing and backoff windows are bounded and verified in policy checks.

AC-3: Exhaustion/fail-closed paths emit stable deterministic reason outputs.

AC-4: Ops configuration docs reflect retry/reconnect envelope governance markers and reasons.

## Conformance Cases

- C-01 (AC-1, Functional): validation lane emits deterministic retry-envelope and reconnect marker set.
- C-02 (AC-1, Regression): policy checker rejects reason taxonomy/version drift with deterministic reason.
- C-03 (AC-2, Functional): policy checker rejects reconnect-attempt bound drift.
- C-04 (AC-2, Functional): policy checker rejects backoff-window bound drift.
- C-05 (AC-3, Regression): policy checker rejects retry-envelope exhaustion fail-closed marker drift.
- C-06 (AC-3, Regression): contract-lane tamper drill rejects with deterministic reason code.
- C-07 (AC-4, Conformance): ops configuration docs and docs-contract tests assert envelope markers/reasons.

## Success Signals

- Retry/reconnect envelope outputs are deterministic across runs.
- Bounded reconnect/backoff limits are enforced by policy checker.
- Exhaustion and envelope drift fail closed with stable reason taxonomy.
