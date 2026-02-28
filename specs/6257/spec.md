# Issue 6257 Spec

Status: Implemented
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6256

## Problem Statement
Five crates identified by the audit are critically under-tested. Current test counts are:
- `kamn-data-layer`: 1
- `kamn-snapshot-journal`: 2
- `kamn-bridges`: 5
- `kamn-crypto`: 10
- `kamn-types`: 2

These low counts leave key validation, parsing, and compatibility behavior insufficiently exercised.

## Scope
In scope:
- Add test cases to each listed crate with deterministic, behavior-focused assertions.
- Increase per-crate test count by at least the observed baseline (+1/+2/+5/+10/+2).
- Keep test additions confined to the five target crates.

Out of scope:
- API redesigns.
- Broad refactors unrelated to enabling/validating tests.
- Coverage policy changes outside touched crates.

## Acceptance Criteria
- AC-1: `kamn-data-layer` test count increases by >=1 (from 1 to >=2).
- AC-2: `kamn-snapshot-journal` test count increases by >=2 (from 2 to >=4).
- AC-3: `kamn-bridges` test count increases by >=5 (from 5 to >=10).
- AC-4: `kamn-crypto` test count increases by >=10 (from 10 to >=20).
- AC-5: `kamn-types` test count increases by >=2 (from 2 to >=4).
- AC-6: All new tests pass under per-crate test runs.

## Conformance Cases
- C-01 (AC-1, Conformance): `rg -n "#\[test\]|proptest!" crates/kamn-data-layer | wc -l` >= 2.
- C-02 (AC-2, Conformance): `rg -n "#\[test\]|proptest!" crates/kamn-snapshot-journal | wc -l` >= 4.
- C-03 (AC-3, Conformance): `rg -n "#\[test\]|proptest!" crates/kamn-bridges | wc -l` >= 10.
- C-04 (AC-4, Conformance): `rg -n "#\[test\]|proptest!" crates/kamn-crypto | wc -l` >= 20.
- C-05 (AC-5, Conformance): `rg -n "#\[test\]|proptest!" crates/kamn-types | wc -l` >= 4.
- C-06 (AC-6, Functional): `cargo test -p kamn-data-layer -p kamn-snapshot-journal -p kamn-bridges -p kamn-crypto -p kamn-types` passes.

## Test Mapping
- Unit: Added per-crate unit tests in each target crate module.
- Functional: Per-crate behavior assertions for parsing/validation/normalization/compatibility paths.
- Conformance: C-01..C-05 count checks + C-06 suite pass.
- Integration: N/A (single-crate scoped behavior tests only).
- Regression: Existing tests in each crate remain green with additions.
- Property/Fuzz/Mutation/Performance: N/A for this task; follow-ups can extend depth further.
