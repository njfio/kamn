# Spec: Issue #6066 - Remove signer adapter key cloning surface

- Issue: #6066
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6067

## Problem Statement
`KolmeForkSecp256k1SignerAdapter` derives `Clone`, which duplicates signing key material in memory and weakens secret-lifecycle guarantees.

## Scope
In scope:
- Remove `Clone` derive from signer adapter.
- Add regression tests to guarantee signing/verification behavior remains intact.
- Add contract test ensuring signer adapter source no longer derives `Clone`.

Out of scope:
- signer API redesign.
- managed signer protocol changes.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: `KolmeForkSecp256k1SignerAdapter` no longer derives `Clone`.
- AC-2: Existing signer adapter functional behavior (sign/verify/public-key derivation) remains green.
- AC-3: Regression guard exists to detect reintroduction of clone derive.

## Conformance Cases
- C-01 (Conformance, AC-1): source contract test asserts signer adapter derive list excludes `Clone`.
- C-02 (Functional, AC-2): existing signer adapter sign/verify tests pass.
- C-03 (Regression, AC-3): clone-derive contract test runs in `kamn-node` test suite.

## Success Metrics / Observable Signals
- Targeted signer test suite remains green after removing derive.
- New contract test fails if clone derive reappears.
