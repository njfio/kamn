# Issue #3914 Plan

- Issue: `#3914`
- Status: `InProgress`

## Approach
- Add unit/regression assertion in signer decode path to ensure raw key input is not echoed in errors.
- Add contract test for source/docs secret-hygiene markers.
- Add CI strategy note for signer redaction regression guard command.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/tests/signer_secret_hygiene_contract.rs`
- `docs/architecture/kolme-runtime-commit.md`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: tests only assert one error surface.
- Mitigation: combine decode regression test with source/docs marker contract checks.

## Interface Contract
- No runtime interface changes.

## ADR
- No ADR required.
