# Objective

Reduce `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` to a thin public wiring surface by extracting residual driver-core construction, MCP live-probe execution, tool-call helpers, and JSON-RPC framing/parsing support into bounded sibling modules without changing MCP-agent runtime behavior.

# Inputs/Outputs

## Inputs
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` at 2541 LOC after inline test extraction
- Existing `kamn-e2e-harness` MCP-agent tests and command contracts
- Existing shared helper validators in `crate::drivers::shared_helpers`
- Existing touched-Rust size policy and file/function size limits

## Outputs
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` reduced to <= 200 LOC
- New bounded sibling modules for the residual root responsibilities, expected seams:
  - `driver_core.rs` for driver construction, execution, and scenario dispatch
  - `live_probe_tranche_one.rs` for S-01 through S-05 live MCP probes
  - `live_probe_tranche_two.rs` for S-06 through S-10 live MCP probes
  - `live_probe_tranche_three.rs` for S-11 through S-15 live MCP probes
  - `tool_call_support.rs` and/or `probe_protocol_support.rs` for shared MCP tool-call and framed JSON-RPC helpers
- Contract coverage that fails if the root grows back above the staged cap or the extracted layout regresses
- Updated spec evidence for the extracted root surface

# Boundaries/Non-goals

- Do not change MCP protocol semantics, scenario behavior, or external command/env contracts
- Do not refactor other drivers in this issue
- Do not add new dependencies

# Failure modes

- `mcp_agent.rs` remains above the 200 LOC cap
- Driver construction, scenario dispatch, or MCP probe bodies stay inline in the root file
- MCP tool-call helpers stop failing hard on spawn, framing, empty output, or non-success responses
- Existing MCP-agent tests or command contracts regress after extraction
- Touched-Rust size policy fails on the issue branch

# Acceptance criteria

- [ ] `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs` is reduced to <= 200 LOC
- [ ] residual root responsibilities are extracted into bounded sibling modules organized by concern
- [ ] real MCP-agent driver wiring remains unchanged
- [ ] a contract test fails if the root grows above the staged cap or the extracted module layout regresses
- [ ] relevant `kamn-e2e-harness` MCP-agent tests and command contracts pass after extraction
- [ ] touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6713-split-mcp-agent-driver-root.md`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent/**`
- `crates/kamn-e2e-harness/tests/**` or `crates/kamn-e2e-harness/src/drivers/**` contract coverage needed for the extraction

# Error semantics

- Extraction preserves the current hard-fail spawn, exit-status, JSON-RPC framing, and payload-validation behavior
- Contract tests fail hard with exact missing-path, file-size, and staged-root-cap details
- No fallback to inline probe implementations or alternate MCP driver wiring layouts

# Test plan

1. Add a red contract that asserts the extracted module layout and a <= 200 LOC root cap.
2. Extract the driver-core, live-probe tranches, and support seams until the contract passes.
3. Run focused `kamn-e2e-harness` MCP-agent tests and command contracts.
4. Run touched-Rust size policy on the issue write set.
