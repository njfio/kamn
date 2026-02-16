# Issue #3913 Plan

- Issue: `#3913`
- Status: `InProgress`

## Approach
- Reuse existing decode-path tests and extend coverage where redaction/marker drift is unguarded.
- Add source/docs contract assertions for zeroization markers.
- Keep runtime behavior unchanged.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/tests/signer_secret_hygiene_contract.rs`
- `docs/architecture/kolme-runtime-commit.md`

## Risks and Mitigations
- Risk: future refactors remove explicit zeroize calls.
- Mitigation: keep marker-based contract tests on critical path.

## Interface Contract
- No runtime interface changes.

## ADR
- No ADR required.
