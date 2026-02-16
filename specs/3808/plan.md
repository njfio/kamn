# Issue #3808 Plan

- Issue: `#3808`
- Status: `InProgress`

## Approach
- Add dedicated contract test target for signer extraction budget/ownership/docs policy.
- Budget check:
  - count `src/signer.rs` LOC
  - fail if above threshold
- Ownership check:
  - assert `mod managed_backend; mod nonce; mod signer_policy;`
  - assert signer public re-exports route through extracted modules
- Docs check:
  - assert `docs/ci/strategy.md` documents guard command and policy markers.

## Affected Modules
- `crates/kamn-node/tests/signer_extraction_budget_contract.rs`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: threshold is too tight and causes churn.
- Mitigation: set threshold above current size with deliberate headroom and explicit remediation text.
- Risk: marker assertions become stale during deliberate refactors.
- Mitigation: keep checks focused on required ownership markers, not incidental implementation text.

## Interface Contract
- No runtime API changes.
- Test/docs guard additions only.

## ADR
- No ADR required: no dependency/protocol changes.
