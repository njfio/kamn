# Plan: #5637 Verify Manifest Nested Field Hardening

## Approach
1. Add RED tests in `command_contract.rs` for missing nested infrastructure/summary markers.
2. Expand `verify_manifest` checks in `src/verify.rs` for required nested fields with deterministic error messages.
3. Re-run verify command tests and full crate gates.
4. Add R56 docs/research marker artifact and docs contract test.

## Affected Modules
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/*docs_contract.rs` (new R56 docs contract)
- `docs/research/`

## Risks and Mitigations
- Risk: brittle string-marker checks in manifest validation.
  - Mitigation: keep deterministic explicit marker list aligned with PRD 8.2 required fields.
- Risk: verify command regression.
  - Mitigation: preserve existing success-path test and report marker assertions.

## Interfaces/Contracts
- No output schema changes.
- Deterministic error messages include missing nested field path marker.
