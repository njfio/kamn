# Issue #5117 Spec

- Title: Task: canonicalize M10 owner scope using KamnDid parser
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M10 compliance projection validates owner DIDs with a local parser but compares requester and owner values as raw strings. Canonical-equivalent owner DIDs can fail authorization due to formatting differences, causing false owner-scope denials.

## Acceptance Criteria
- AC-1: M10 owner DID parsing uses canonical `KamnDid::parse` in projection authorization paths.
- AC-2: Owner-scope comparison is performed on canonical DID strings.
- AC-3: Canonical-equivalent owner DID inputs succeed while truly different owners remain fail-closed.
- AC-4: Existing M10 behavior remains deterministic and full M10 tests pass.

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `specs/5117/{spec.md,plan.md,tasks.md}`

Out of scope:
- M10 archival lifecycle semantics changes
- New dependencies
- Non-M10 module refactors

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Projection request with canonical-equivalent requester/owner DIDs | Authorization succeeds via canonical parse path |
| C-02 | AC-2 | Conformance | Requester DID with whitespace, owner DID canonical | No false owner-scope denial |
| C-03 | AC-3 | Conformance | Requester and owner DIDs from different owners | Fail-closed with owner-scope reason code |
| C-04 | AC-4 | Regression | Existing + new M10 conformance tests | All pass with deterministic outputs |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m10_partition_archival -- spec_c08_partition_projection_accepts_canonical_equivalent_owner_dids`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival -- spec_c09_partition_projection_denies_non_equivalent_owner_dids`
- `cargo test -p kamn-core --test data_layer_m10_partition_archival`
- `cargo test -p kamn-core --test data_layer_m10_partition_recoverability`
- `cargo clippy -p kamn-core -- -D warnings`
- `cargo fmt --check`

## Success Metrics
- Canonical-equivalent owner DID inputs no longer cause false projection denials.
- Non-equivalent owner scopes remain fail-closed.
