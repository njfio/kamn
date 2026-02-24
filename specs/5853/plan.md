# Plan: Issue #5853

## Approach
1. Extract CLI runtime execution flow into a testable helper that accepts parsed `NodeCli` so full-mode supervisor behavior can be exercised directly in tests.
2. Add a full-mode supervisor branch in the CLI runtime path:
   - build endpoint configs
   - enforce deterministic lane contract checks for full-mode endpoint lanes
   - start endpoint lanes under supervisor handling
   - execute daemon/full report path
   - join lane handles and fail closed on lane errors with stable reason codes
3. Add endpoint lane self-probe behavior for full-mode supervisor default lane budgets (`max_requests=1`) so lanes complete deterministically in tests and CLI runs.
4. Keep legacy non-full runtime path behavior unchanged.
5. Add/adjust runtime tests in full-supervisor test surface for lane-order and lane-contract fail-closed assertions.

## Affected Modules
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/main_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/full_supervisor_and_shutdown_tests.rs`
- `specs/5853/spec.md`
- `specs/5853/plan.md`
- `specs/5853/tasks.md`

## Risks & Mitigations
- Risk: full-mode endpoint lanes time out and cause nondeterministic test failures.
  - Mitigation: explicit full-mode lane contract checks + supervisor self-probe on default one-request lane budgets.
- Risk: regression to existing full-supervisor stop reason-code contracts.
  - Mitigation: preserve existing stop validation path and run targeted regression tests.
- Risk: introducing behavior drift for non-full runtime modes.
  - Mitigation: isolate new behavior to full-mode supervisor branch only.

## Interfaces / Contracts
- New fail-closed reason-code contract markers in `ConfigError::RuntimeDaemonLifecycle` for:
  - service-api lane max-request contract drift
  - observability lane max-request contract drift
  - lane thread join/serve failures (deterministic lane identifiers)
- Existing full-supervisor stop contract markers remain unchanged.

## ADR
- Not required (no dependency/protocol/architecture decision beyond local runtime orchestration wiring).
