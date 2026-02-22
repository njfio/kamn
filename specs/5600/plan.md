# Issue #5600 Plan - PRD Phase-6 Runtime External Execution Integration

## Approach
1. Add RED tests for parser flag support, runtime integration output markers, and deterministic preflight failures.
2. Implement `RunCommandConfig.external_execution` plus parser handling for `--enable-external-execution`.
3. Implement guarded preflight checks in `execute_run_contract` and emit `runtime_external_execution` contract object.
4. Add phase-6 runtime integration docs marker artifact and milestone progression update.
5. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase6_runtime_integration_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase6-runtime-integration-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: introducing execution toggle regresses existing deterministic tests.
  - Mitigation: default `external_execution=false` and keep prior behavior unchanged.
- Risk: preflight checks are ambiguous.
  - Mitigation: deterministic explicit error strings per failed precondition.

## Interfaces / Contracts
- `run --enable-external-execution`
- `runtime_external_execution.requested`
- `runtime_external_execution.guard_status`
- `runtime_external_execution.execution_mode`
- `runtime_external_execution.preflight`

## ADR
- Not required for this additive guarded integration contract slice.
