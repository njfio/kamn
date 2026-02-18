# Issue #3653 Plan

- Issue: `#3653`
- Status: `Completed`

## Approach
- Move key-source and crypto logic under signer adapter module boundaries.
- Keep caller-facing behavior unchanged.
- Enforce module ownership and parity with dedicated contracts.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/`
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `scripts/kolme/`

## Risks and Mitigations
- Risk: behavior drift in signing outputs.
- Mitigation: signature parity matrix + policy checks.
- Risk: boundary regressions over time.
- Mitigation: adapter boundary and extraction budget tests.

## Interface Contract
- Signer runtime call behavior remains stable.

## ADR
- No ADR required.
