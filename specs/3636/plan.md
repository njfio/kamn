# Issue #3636 Plan

- Issue: `#3636`
- Status: `Completed`

## Approach
- Consolidate signer crypto + key-source operations under adapter module boundaries.
- Re-export only required call surfaces to keep runtime callers stable.
- Enforce ownership and parity with contract tests and lane checks.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/`
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `crates/kamn-node/tests/signer_extraction_budget_contract.rs`
- `scripts/kolme/`

## Risks and Mitigations
- Risk: accidental API visibility drift.
- Mitigation: adapter boundary contract tests.
- Risk: signing behavior changes.
- Mitigation: signature parity contract/policy lanes.

## Interface Contract
- Runtime signing call behavior remains unchanged.
- Adapter internals become module-owned implementation detail.

## ADR
- No ADR required.
