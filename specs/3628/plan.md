# Issue #3628 Plan

- Issue: `#3628`
- Status: `Completed`

## Approach
- Execute decomposition through child tasks (`#3636`, `#3637`, `#3638`) and subtasks.
- Keep signer public contract stable while moving responsibilities into focused modules.
- Enforce stability via parity lanes and drift-checking contracts.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/`
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `crates/kamn-node/tests/signer_extraction_budget_contract.rs`
- `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- `scripts/kolme/`

## Risks and Mitigations
- Risk: behavior drift during module extraction.
- Mitigation: parity contract lanes and deterministic reason taxonomy tests.
- Risk: ownership drift back into root signer file.
- Mitigation: extraction budget/boundary contracts.

## Interface Contract
- Runtime signer-facing behavior remains stable.
- Module internals are reorganized behind existing runtime call paths.

## ADR
- No ADR required (decomposition with preserved external behavior).
