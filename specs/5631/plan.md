# Plan: #5631 TEARDOWN Phase Activation

## Approach
1. Add RED tests in `crates/kamn-e2e-harness/tests/command_contract.rs` for teardown phase semantics across sdk and mcp modes plus lifecycle total propagation.
2. Implement teardown step/status/detail activation in `crates/kamn-e2e-harness/src/lib.rs`:
   - PRD step inventory for phase 5 teardown.
   - mode-aware MCP shutdown step status.
   - deterministic teardown detail summarization.
3. Re-run harness contract tests and full crate gates; update docs/research marker for R54 teardown activation.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/*docs_contract.rs` (new docs contract test)
- `docs/research/`

## Risks and Mitigations
- Risk: Lifecycle count drift across many existing assertions.
  - Mitigation: Update only assertions coupled to teardown semantics and re-run full `command_contract` suite.
- Risk: MCP/non-MCP mode regressions.
  - Mitigation: Add explicit mode-specific teardown conformance tests.
- Risk: Orchestration/live status regressions.
  - Mitigation: keep orchestration status derivation logic unchanged, rely on existing live status tests.

## Interfaces/Contracts
- No schema shape changes; only semantic activation of existing `phase_results` teardown fields.
- No new public API surface.
