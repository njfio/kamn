# Issue #3638 Plan

- Issue: `#3638`
- Status: `InProgress`

## Approach
- Complete migration parity via focused child slices:
  - `#3766`: parity matrix + drift guard
  - `#3808`: size/ownership budget guard
- Keep signer runtime behavior unchanged; add contract tests/docs where gaps exist.
- Run signer-focused verification after each slice.

## Affected Modules
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-node/tests/*` signer migration docs/source contract tests
- `docs/architecture/signer-lifecycle.md`
- `docs/ci/strategy.md` (for budget guard follow-up in `#3808`)

## Risks and Mitigations
- Risk: parity coverage misses edge selector paths.
- Mitigation: matrix includes primary/secondary + env-local/managed-external contracts and explicit failure paths.
- Risk: docs drift from source markers.
- Mitigation: docs/source contract tests fail closed on missing markers.
- Risk: monolith regrowth post-migration.
- Mitigation: add budget guard in `#3808`.

## Interface Contract
- No public runtime API changes.
- Additional test/docs contracts only.

## ADR
- No ADR required: migration completion introduces no protocol/dependency changes.
