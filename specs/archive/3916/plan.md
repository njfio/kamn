# Issue #3916 Plan

- Issue: `#3916`
- Status: `Completed`

## Approach
- Define required lifecycle marker set constants in test policy helper.
- Reject forbidden fallback reason code and any missing/blank marker value.
- Keep checks deterministic and side-effect free.

## Affected Modules
- `crates/kamn-node/tests/signer_secret_lifecycle_policy_contract.rs`

## Risks and Mitigations
- Risk: required marker list diverges from runtime report contract.
- Mitigation: pin marker list in docs and contract test assertions.

## Interface Contract
- No runtime interface changes.

## ADR
- No ADR required.
