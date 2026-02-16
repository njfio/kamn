# Issue #3811 Plan

- Issue: `#3811`
- Status: `Completed`

## Approach
- Add a dedicated signer adapter boundary contract test target.
- Extract adapter-owned symbols from `signer.rs` into new `signer_adapter` module.
- Re-export adapter API from `signer.rs` and remove inlined adapter implementation from root module.
- Add architecture docs markers and guard command for adapter boundary parity.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/signer_adapter.rs`
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `docs/architecture/kolme-runtime-commit.md`

## Risks and Mitigations
- Risk: visibility changes break existing signer tests.
- Mitigation: preserve public crate-level API surface through explicit re-exports and scoped regression suites.
- Risk: managed backend hex helper references drift when moved.
- Mitigation: keep helper re-export compatibility from `signer.rs`.

## Interface Contract
- No external API changes.
- Internal module ownership and contract-test surface only.

## ADR
- No ADR required.
