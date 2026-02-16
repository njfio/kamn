# Issue #3912 Plan

- Issue: `#3912`
- Status: `InProgress`

## Approach
- Keep existing signer behavior intact.
- Add explicit regression for decode-failure redaction safety in signer tests.
- Add source/docs contract test for zeroization and hygiene markers.
- Update `docs/architecture/kolme-runtime-commit.md` with decode-path zeroization guarantees.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/tests/signer_secret_hygiene_contract.rs`
- `docs/architecture/kolme-runtime-commit.md`

## Risks and Mitigations
- Risk: regression assertions are too weak and miss leakage drift.
- Mitigation: assert sensitive input token is absent from error surfaces.
- Risk: docs drift from implementation.
- Mitigation: docs/source contract test with required markers.

## Interface Contract
- No public API changes.
- Test/docs guarantee hardening only.

## ADR
- No ADR required: no new dependency or protocol decision.
