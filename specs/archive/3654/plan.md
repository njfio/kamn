# Issue #3654 Plan

- Issue: `#3654`
- Status: `Completed`

## Approach
- Keep policy logic in signer policy module with deterministic reason-code contracts.
- Preserve runtime integration behavior via signer policy/emulator lanes.
- Add drift checks for fallback marker taxonomy.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/`
- `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- `scripts/signer/`
- `scripts/kolme/`

## Risks and Mitigations
- Risk: reason taxonomy drift.
- Mitigation: dedicated signer policy taxonomy contract.
- Risk: runtime behavior regression.
- Mitigation: signer emulator/policy lanes and fallback marker checks.

## Interface Contract
- Signer policy decision behavior remains deterministic and fail closed.

## ADR
- No ADR required.
