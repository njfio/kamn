# Issue #7143: Repair Pi Extension Claim Contract After MCP Module Split

## Objective

Restore the project-local Pi extension claim contract after live MCP configuration was
extracted into its own TypeScript module. The contract must inspect the delegated
configuration source before asserting the environment markers that define the live MCP
tool boundary.

## Inputs And Outputs

### Inputs

- The explicit project-local KAMN MVP Pi extension source inventory.
- `.pi/extensions/kamn-mvp/mcp-session-config.ts`.
- The live MCP environment marker assertions in
  `spec_c08_project_local_pi_extension_registers_kamn_tools`.

### Outputs

- A source bundle that includes the delegated MCP session configuration module.
- A passing Pi extension marker contract on current `main`.
- A regression assertion that binds the source inventory to the delegated module path.

## Boundaries And Non-Goals

- Do not change Pi runtime behavior, tool registration, or MCP session semantics.
- Do not add dependencies or discover extension files dynamically.
- Do not change public APIs or environment-variable requirements.
- Do not resolve repository-wide governance-ratio or unrelated #7126 closeout gates.
- Do not perform another live devnet settlement.

## Failure Modes

- The claim contract omits a delegated TypeScript module and falsely reports that a
  required runtime marker is missing.
- The delegated module is renamed or removed without updating the explicit contract
  inventory.
- A missing source file is silently ignored instead of failing the contract.

## Acceptance Criteria

- [x] The source inventory includes
  `.pi/extensions/kamn-mvp/mcp-session-config.ts`.
- [x] The contract asserts that the delegated module path remains in the inventory.
- [x] `spec_c08_project_local_pi_extension_registers_kamn_tools` passes.
- [x] The complete `mvp_demo_agent_harness_claim_contract` test target passes.
- [x] No production Pi extension source changes.

## Files To Touch

- `specs/7143-pi-extension-claim-contract.md`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`

## Error Semantics

- Missing inventoried source files continue to panic with the exact missing path.
- Missing tool markers continue to panic with the exact missing marker.
- No fallback, directory-wide discovery, or ignored read error is allowed.

## Test Plan

### RED

- Reproduce the inherited failure for
  `spec_c08_project_local_pi_extension_registers_kamn_tools`, which reports the missing
  `KAMN_MVP_LIVE_MCP_BINARY` marker.
- Add a contract assertion requiring the delegated MCP configuration path in the source
  inventory; verify that it fails before the inventory is updated.

### GREEN

- Add `mcp-session-config.ts` to the explicit source inventory.
- Run the focused marker test and the complete claim-contract test target.

### REFACTOR

- Keep the source inventory explicit, ordered with the related MCP session modules, and
  free of duplicated path literals.
- Run formatting and Clippy for the touched Rust test target.

### INTEGRATION

- Run the real `kamn-e2e-harness` claim-contract integration test target from a clean
  checkout.

## Verification Evidence

- `cargo fmt --all -- --check`
- `cargo clippy -p kamn-e2e-harness --tests -- -D warnings`
- `cargo test -p kamn-e2e-harness --test mvp_demo_agent_harness_claim_contract`
  passes all 24 tests.
- Both touched Rust test files remain below 200 lines.
