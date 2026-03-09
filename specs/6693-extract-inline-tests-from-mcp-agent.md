# 6693 Extract Inline Tests From MCP Agent Driver

## Objective

Move the inline `#[cfg(test)]` surface out of `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` into bounded sibling test modules so the production driver file stops carrying test-only helpers and shrinks to a staged target focused on runtime logic.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
  - existing MCP-agent driver tests currently embedded in the inline `#[cfg(test)]` module
- Outputs:
  - a reduced `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` with production logic only
  - extracted test modules under `crates/kamn-e2e-harness/src/drivers/`
  - contract coverage proving the inline test surface moved and the root budget ratcheted downward

## Boundaries/Non-goals

- Do not change MCP-agent runtime behavior or scenario semantics
- Do not rewrite live probe logic beyond import/path adjustments required by the move
- Do not weaken or delete existing test coverage to satisfy the split
- Do not attempt a full production-logic decomposition of `mcp_agent.rs` in this issue

## Failure Modes

- Production code loses access to helper items because test-only imports are moved incorrectly
- Extracted tests stop compiling because they no longer reach private driver helpers
- The root file still keeps the inline `#[cfg(test)]` module after the move
- New extracted test files exceed the file budget without a staged split contract
- Contract coverage passes without proving the inline test surface actually moved

## Acceptance Criteria

- [ ] Remove the inline `#[cfg(test)]` module from `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- [ ] Extract the MCP-agent test surface into sibling modules with logical seams and `<= 200` LOC leaf files
- [ ] Reduce `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` to a staged target of `<= 2600` LOC
- [ ] Add or update a contract test that proves the inline test module moved and the root file budget ratcheted downward
- [ ] Preserve the existing MCP-agent driver test entrypoints and assertions
- [ ] `cargo test -p kamn-e2e-harness mcp_agent -- --nocapture` passes

## Files To Touch

- `specs/6693-extract-inline-tests-from-mcp-agent.md`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- new extracted MCP-agent test modules under `crates/kamn-e2e-harness/src/drivers/`
- new or updated extraction contract coverage for the MCP-agent driver

## Error Semantics

- Test-only extraction must preserve existing fail-closed assertion behavior
- No production-path fallbacks or silent error handling may be introduced
- Helper visibility changes must remain scoped to test compilation boundaries only

## Test Plan

1. Add a red contract that requires the inline `#[cfg(test)]` module to leave `mcp_agent.rs`, requires new extracted MCP-agent test modules, and ratchets the root file to `<= 2600` LOC.
2. Run the contract and confirm it fails before implementation.
3. Extract the inline MCP-agent test surface into bounded sibling modules organized by concern.
4. Re-run:
   - `cargo test -p kamn-e2e-harness mcp_agent -- --nocapture`
   - targeted MCP-agent extraction contract coverage
5. Run the touched-Rust size policy on the final write set.
