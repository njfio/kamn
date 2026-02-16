# Issue #3766 Plan

- Issue: `#3766`
- Status: `InProgress`

## Approach
- Add a signer migration parity matrix functional test in signer test ownership.
- Add a dedicated docs parity contract test for `docs/architecture/signer-lifecycle.md`.
- Update signer lifecycle docs with parity matrix markers and guard command references.
- Keep runtime behavior unchanged and validate with scoped signer suites.

## Affected Modules
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-node/tests/signer_migration_parity_docs_contract.rs`
- `docs/architecture/signer-lifecycle.md`

## Risks and Mitigations
- Risk: matrix misses one supported contract combination.
- Mitigation: explicitly include primary/env-local, secondary/env-local, primary/managed-external and disallowed secondary/managed-external.
- Risk: docs drift from test/source behavior.
- Mitigation: enforce required markers via docs parity contract tests.

## Interface Contract
- No public API changes.
- Test/docs guard additions only.

## ADR
- No ADR required: no dependency/protocol changes.
