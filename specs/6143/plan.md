# Plan: Issue #6143

## Approach
1. Capture RED evidence that no inter-tick lane health probe helper exists in runtime orchestration.
2. Add an inter-tick probe helper for full-supervisor lanes that:
   - probes service-api and observability lanes once during daemon runtime
   - fails closed on non-success responses
3. Wire helper into `execute_full_supervisor_daemon_runtime` loop after lane liveness state checks.
4. Update full-supervisor lane `max_requests` internal budgets to account for startup + inter-tick + shutdown probes.
5. Add regression/unit coverage for one-shot probe behavior and fail-closed probe error path.
6. Run scoped `kamn-node` tests for new and related lane-liveness paths.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration.rs`
- `specs/6143/spec.md`
- `specs/6143/plan.md`
- `specs/6143/tasks.md`

## Risks / Mitigations
- Risk: Added probes consume lane request budget and trigger early lane completion.
  Mitigation: increase internal lane max-request allowance to cover startup/inter-tick/shutdown probes.
- Risk: Inter-tick probes introduce runtime overhead.
  Mitigation: execute probes once per lane during runtime, not per tick.
- Risk: Probe helper drift reintroduces startup-only behavior.
  Mitigation: add deterministic regression tests for one-shot execution and fail-closed errors.

## Interfaces / Contracts
- No external API/wire contract changes.
- Internal full-supervisor runtime/liveness contract behavior only.
