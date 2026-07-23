# Issue #7156: Bind MCP Settlement Authority Fixture

## Objective

Align the MCP `release_escrow` dispatch success fixture with the service-backed
settlement receipt authority contract.

## Inputs And Outputs

- Input: a release authorization service receipt plus a distinct confirmed settlement
  service receipt.
- Output: a successful MCP authority envelope that preserves both receipt authorities.

## Boundaries And Non-Goals

- Do not change production authority parsing or verification.
- Do not weaken missing, malformed, partial, tampered, or released-state rejection.
- Do not change public MCP tool schemas or error codes.
- Do not change escrow, Pi, runtime receipt-chain, or governance behavior.

## Failure Modes

- The success fixture reports the terminal `released` state as primary authority.
- The success fixture omits confirmed settlement receipt fields.
- The settlement receipt resource differs from the requested escrow.
- Negative authority cases begin passing.

## Acceptance Criteria

- [ ] The release success fixture reports `release-authorized` primary state.
- [ ] The fixture includes a distinct `settlement:confirmed` service receipt.
- [ ] The successful response retains the release-authorized service result.
- [ ] Existing negative authority cases remain rejected.
- [ ] The complete tool-dispatch and authority-receipt targets pass.
- [ ] Formatting and strict Clippy pass.

## Files To Touch

- `specs/7156-bound-mcp-settlement-authority.md`
- `crates/kamn-mcp-server/tests/tool_dispatch_contract.rs`

## Error Semantics

- Complete bound release and settlement authority succeeds.
- Missing, malformed, partial, tampered, mismatched, or terminal primary authority
  returns `MCP_AUTHORITY_RECEIPT_MISSING` or `MCP_AUTHORITY_RECEIPT_INVALID`.

## Test Plan

### RED

- Isolate the stale release success fixture and reproduce
  `MCP_AUTHORITY_RECEIPT_INVALID`.

### GREEN

- Bind the fixture to release authorization and confirmed settlement receipts.

### REFACTOR

- Extract the bound release fixture if needed to keep the backend readable.

### INTEGRATION

- Run the complete tool-dispatch and authority-receipt targets, formatting, strict
  Clippy, and the full workspace gate.
