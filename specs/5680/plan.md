# Plan: #5680 External Runtime Probe Execution in E2E Harness

## Approach
1. Add RED conformance tests that assert external marker blocks are probe-derived and encode failures for non-zero probe exits.
2. Introduce an internal runtime-probe model in `kamn-e2e-harness/src/lib.rs` to execute configured binaries (safe probe invocation) and aggregate component statuses.
3. Wire probe results into `runtime_orchestration`, `runtime_lifecycle_execution`, and `runtime_validation_execution` JSON outputs for external execution mode.
4. Keep external-disabled path and preflight validation behavior intact.
5. Update PRD implementation progress notes for this slice and run crate-level verification gates.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/prd/e2e-live-testing-prd.md`

## Risks and Mitigations
- Risk: Probe command assumptions could make tests flaky across environments.
- Mitigation: Use temporary executable scripts in tests for deterministic exit behavior and keep probe command minimal.

- Risk: JSON marker changes could break existing contract tests.
- Mitigation: Update only the external-enabled expectations; preserve non-external marker shape and content.

## Interfaces / Contracts
- Existing JSON keys preserved:
  - `runtime_orchestration`
  - `runtime_lifecycle_execution`
  - `runtime_validation_execution`
- External-enabled marker semantics change from static scaffold to probe-derived status.

## ADR
- Not required. No dependency, protocol, or architecture boundary change.
