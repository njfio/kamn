# Plan: #5693 Mutation Hardening for `kamn-mcp-server` Protocol Helpers

## Approach
1. Add helper-focused unit tests inside `protocol.rs` to cover escaped branches.
2. Keep tests deterministic and narrow to escaped mutant regions.
3. Run RED/GREEN on targeted tests and full crate tests.
4. Rerun mutation gate on in-diff scope and capture improved telemetry.

## Affected Modules
- `crates/kamn-mcp-server/src/protocol.rs`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract.rs` (only if needed)

## Risks and Mitigations
- Risk: helper tests overfit string layout details.
  Mitigation: assert behavior-level outcomes (result class and key markers), not
  fragile full-string equality where avoidable.

- Risk: mutation misses persist due equivalent mutants.
  Mitigation: explicitly document remaining misses with rationale if equivalent
  or non-actionable.

## Interfaces / Contracts
- No public API or CLI surface changes.
- No protocol shape changes; this slice strengthens test depth only.

## ADR
- Not required. No architecture or dependency change.
