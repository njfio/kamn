# Issue #5322 Plan

## Approach
1. Introduce a crate-wide `#[cfg(test)]` signer env lock accessor in `main.rs`.
2. Route `main_tests` signer lock helper through that shared accessor.
3. Route `signer.rs` test lock helper through the same shared accessor.
4. Keep existing poison-recovery helper semantics in `main_tests` and ensure no reason-code behavior changes.
5. Verify targeted regression tests with parallel test threads.

## Affected Modules
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/main_tests.rs`
- `crates/kamn-node/src/signer.rs`
- `specs/5322/spec.md`
- `specs/5322/plan.md`
- `specs/5322/tasks.md`

## Risks and Mitigations
- Risk: introducing deadlock by nested lock acquisition.
  - Mitigation: preserve existing lock acquisition points and only change lock source.
- Risk: unintended behavior drift in signer tests.
  - Mitigation: run targeted regression and reason-code assertions unchanged.

## Interfaces and Contracts
- Test-only contract: all signer-env-mutating tests must rely on one crate-wide lock domain.
- Production interfaces unchanged.
