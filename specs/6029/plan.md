# Plan: Issue #6029

## Approach
1. Add test fixtures for deterministic owner/agent DIDs, M9 registry setup, channel membership, and anti-spam admission.
2. Write RED tests for unsupported transport, dispatch projection, and presence projection behavior.
3. Keep production code unchanged unless tests expose deterministic contract defects.
4. Run focused `kamn-core` module tests, then adjacent M9 module tests for regression confidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_m9_gateway_bridge.rs`

## Risks / Mitigations
- Risk: dispatch path can fail due to channel-membership or anti-spam preconditions rather than projection behavior.
  Mitigation: construct minimal valid channel + anti-spam fixtures and assert only contract-level outputs.
- Risk: presence visibility policy may deny queries unexpectedly.
  Mitigation: use requester==target queries for deterministic visibility and explicit disconnected-target case.

## Interfaces / Contracts
- No public API changes.
- Test-only coverage additions validating existing projection contracts.
