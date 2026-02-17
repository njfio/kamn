# Plan — Issue #4165

## Approach

1. Add a failing unit test covering signer secret precedence failure with an in-memory secret buffer.
2. Add a regression marker test ensuring the precedence path continues to include explicit zeroization.
3. Keep signer precedence reason-code assertions intact to avoid behavior drift.

## Affected Modules

- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`

## Risks / Mitigations

- Risk: test-only coverage could miss runtime path integration.
  Mitigation: keep existing functional precedence failure tests and run both unit + functional assertions.
- Risk: brittle source-marker assertions.
  Mitigation: limit marker checks to stable, intentional zeroization hooks.

## Interfaces / Contracts

- Preserve precedence failure reason code:
  - `signer_secret_source_precedence_violation`
- Preserve strict signer profile/key-source contract behavior.

## ADR

No ADR required. No dependency/protocol boundary changes.

