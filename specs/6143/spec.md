# Spec: Issue #6143 - Task: [X-05] Add daemon inter-tick lane liveness monitoring

- Issue: #6143
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6099

## Problem Statement
Full-supervisor lanes are probed at startup and shutdown, but there is no explicit inter-tick health probe execution contract while daemon runtime is in flight. This leaves an execution-window blind spot for lane health validation.

## Scope
In scope:
- Add explicit inter-tick lane health probe execution during full-supervisor daemon runtime.
- Preserve fail-closed behavior if inter-tick probe returns non-success response.
- Update full-supervisor lane request budgets to allow startup + inter-tick + shutdown probes.
- Add regression/conformance tests for one-shot inter-tick probing and fail-closed probe errors.

Out of scope:
- Deep refactors of lane serving implementations.
- Protocol/schema changes outside supervisor liveness behavior.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Full-supervisor daemon execution performs explicit lane health probes during daemon runtime (not startup/shutdown only).
- AC-2: Inter-tick probe failures fail closed with deterministic reason-code classification.
- AC-3: Regression/conformance tests verify one-shot inter-tick probing behavior and fail-closed error handling.

## Conformance Cases
- C-01 (AC-1, Functional/Conformance): `execute_full_supervisor_daemon_runtime` invokes inter-tick lane health probes while daemon handle is still active.
- C-02 (AC-2, Regression): Non-success inter-tick probe response returns `RuntimeDaemonLifecycle` with deterministic `http_status` classification.
- C-03 (AC-3, Unit/Conformance): Inter-tick probe helper executes at most once per lane and marks completion flags deterministically.

## Success Metrics / Observable Signals
- Targeted R59 finding `X-05` no longer appears as unresolved in follow-up review docs.
- Required scoped test commands pass in CI and local verification runs.
- Closure comment includes deterministic evidence links and tier coverage summary.
