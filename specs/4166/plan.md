# Plan — Issue #4166

## Approach

1. Add/adjust signer secret precedence helper to scrub env-secret buffers before error return.
2. Scrub transient key-material buffers during managed signing key construction path.
3. Update ops/threat docs and lock them with docs-contract tests.

## Affected Modules

- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/signer_adapter.rs`
- `docs/ops/configuration.md`
- `docs/foundation/threat-control-matrix.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `crates/kamn-core/tests/threat_control_matrix_docs.rs`

## Risks / Mitigations

- Risk: behavior drift in strict signer precedence flow.
  Mitigation: preserve existing precedence violation tests and reason-code assertions.
- Risk: docs-policy drift after code changes.
  Mitigation: add targeted docs-contract tests for zeroization markers.

## Interfaces / Contracts

- Preserve signer precedence fail-closed reason:
  - `signer_secret_source_precedence_violation`
- Preserve existing signer adapter parse fail/success behavior while tightening memory hygiene.

## ADR

No ADR required. No dependency/protocol changes.

