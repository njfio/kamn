# Issue #5610 Plan - External Execution Preflight Executable Diagnostics

## Approach
1. Add RED tests for non-executable and executable binary preflight behavior.
2. Implement executability checks in preflight with deterministic error messages.
3. Add docs/milestone markers for R52 slice tracking.
4. Run required quality gates.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/r52_preflight_executable_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-r52-preflight-executable-diagnostics.md` (new)
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: platform-specific executability checks diverge.
  - Mitigation: use `cfg(unix)` permission-bit checks and deterministic unsupported-path handling for non-unix.
- Risk: regressions in existing phase-6 contracts.
  - Mitigation: run full harness suite and cross-crate regressions.

## Interfaces / Contracts
- `ensure_external_execution_preflight` returns deterministic errors for:
  - non-executable `kolme_binary`
  - non-executable `agent_binary` in MCP modes

## ADR
- Not required (behavior hardening within existing preflight contract).
