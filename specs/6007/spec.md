# Spec: Issue #6007 - Replace residual synthetic daemon/metrics behavior with runtime-driven processing

- Issue: #6007
- Status: Reviewed
- Type: story
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-26
- Parent: #5917
- Follow-up: #5895

## Problem Statement
Audit follow-up still reports synthetic behavior in runtime processing and observability signals. The daemon/service integration must prove relay lifecycle progression and metric counter advancement from live runtime execution paths, not projection-only or placeholder behavior.

## Scope
In scope:
- Enforce runtime relay processing during daemon tick execution in full runtime paths.
- Ensure service API runtime observability counters for relay progression are incremented from live processing outcomes.
- Add conformance-focused tests that prove send -> tick-loop processing -> observable state and metric progression.

Out of scope:
- WebSocket persistent stream redesign.
- New transport protocols.
- Shell/workflow/governance process changes.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Full runtime mode executes relay processing each daemon tick and records deterministic drained/projected totals in daemon execution output.
- AC-2: Service API observability metrics expose live relay progression counters that advance after runtime relay processing.
- AC-3: Integration tests prove runtime lifecycle progression from accepted send to relayed/delivered observable state.
- AC-4: Existing full-supervisor fail-closed and startup/stop contract tests remain green.

## Conformance Cases
- C-01 (Unit, AC-1): daemon relay processing function returns deterministic drained/projected counters for queued relay entries.
- C-02 (Functional, AC-1/AC-2): full runtime execution emits non-zero relay counters after queued relay work is processed.
- C-03 (Integration, AC-2/AC-3): service API metrics and message query outputs reflect runtime relay progression after daemon tick execution.
- C-04 (Regression, AC-4): full-supervisor lane liveness and stop marker contracts remain unchanged and green.

## Success Metrics / Observable Signals
- Runtime report fields `service_api_relay_drained_count` and `service_api_relay_projected_state_count` advance from live processing in positive-path integration runs.
- `/metrics` relay counters map to live runtime state and are non-placeholder in integration tests that drive relay work.
- Full-supervisor integration/regression contracts remain green.
