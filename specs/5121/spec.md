# Issue #5121 Spec

- Title: Task: canonicalize M8 owner DID keys using KamnDid parser
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M8 owner DID handling uses local DID parsing with raw owner keying/comparison. Canonical-equivalent owner DIDs can validate but still fail lookup or owner-scope authorization due to string formatting differences.

## Acceptance Criteria
- AC-1: M8 owner DID parsing uses canonical `KamnDid::parse` paths.
- AC-2: Owner-scoped keying and owner-scope comparisons are canonicalized.
- AC-3: Canonical-equivalent owner DIDs succeed for lookup/authorization paths.
- AC-4: Existing deterministic M8 behavior remains stable and full M8 suite passes.

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs`
- `crates/kamn-core/tests/data_layer_m8_compliance_lifecycle.rs`
- `specs/5121/{spec.md,plan.md,tasks.md}`

Out of scope:
- M8 retention/shredding semantic changes
- New dependencies
- Non-M8 module refactors

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | M8 register + owner APIs with canonical owner DID | Canonical parser path used and behavior deterministic |
| C-02 | AC-2 | Conformance | Owner-scope operations with whitespace-wrapped canonical-equivalent owner DID | Scope resolution/authorization succeeds |
| C-03 | AC-3 | Conformance | Message lookup and retention projections with canonical-equivalent owner DID | No false `OwnerNotFound`/scope denial |
| C-04 | AC-4 | Regression | Existing + new M8 conformance tests | All pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle -- spec_c11_message_lookup_accepts_canonical_equivalent_owner_did`
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle -- spec_c12_owner_scope_accepts_canonical_equivalent_requester_owner_did`
- `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Canonical-equivalent owner DID inputs no longer fail owner-scoped lookup/authorization in M8.
- Full M8 conformance suite remains green and deterministic.
