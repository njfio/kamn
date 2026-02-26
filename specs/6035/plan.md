# Plan: Issue #6035

## Approach
1. Add deterministic fixture helpers for owner/message/wrapped-key registration.
2. Write RED tests for legal-hold fail-closed shred behavior and due-candidate filtering/order.
3. Keep production code unchanged unless tests expose contract mismatch.
4. Run focused M8 module tests plus adjacent M7/M9 slices for regression confidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs`

## Risks / Mitigations
- Risk: retention windows are long constants and can make due-cutoff assertions brittle.
  Mitigation: use explicit timestamps with large deterministic `now_epoch_seconds` values.
- Risk: fixture noise can mask owner-scope behavior.
  Mitigation: use small explicit owner/message sets and assert error variants/reason codes directly.

## Interfaces / Contracts
- No public API changes.
- Test-only additions validating existing M8 registry contracts.
