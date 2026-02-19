# Issue #5119 Spec

- Title: Task: canonicalize M5 owner DID keys using KamnDid parser
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M5 owner DID validation is local and owner map keys/lookups use raw input strings. Canonical-equivalent owner DIDs can validate but still miss owner scope lookups, causing false `OwnerNotFound` outcomes.

## Acceptance Criteria
- AC-1: M5 owner DID parsing uses canonical `KamnDid::parse` paths.
- AC-2: Owner-scoped map keys and lookup/query paths use canonical owner DID strings.
- AC-3: Canonical-equivalent owner DID inputs succeed for retention/query lookups.
- AC-4: Existing deterministic M5 behavior remains stable and full M5 tests pass.

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs`
- `crates/kamn-core/tests/data_layer_m5_vector_integration.rs`
- `specs/5119/{spec.md,plan.md,tasks.md}`

Out of scope:
- M5 ranking/anomaly semantics changes
- New dependencies
- Non-M5 module refactors

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | M5 append + query with canonical owner DID | Canonical parser path is used and operations succeed deterministically |
| C-02 | AC-2 | Conformance | Owner lookup/query with whitespace-wrapped equivalent DID | Existing owner scope is resolved (no false `OwnerNotFound`) |
| C-03 | AC-3 | Conformance | Retention projection and semantic query using canonical-equivalent owner DID | Success with deterministic rows |
| C-04 | AC-4 | Regression | Existing + new M5 conformance suite | All pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m5_vector_integration -- spec_c12_retention_due_accepts_canonical_equivalent_owner_did`
- `cargo test -p kamn-core --test data_layer_m5_vector_integration -- spec_c13_semantic_query_accepts_canonical_equivalent_owner_did`
- `cargo test -p kamn-core --test data_layer_m5_vector_integration`
- `cargo clippy -p kamn-core -- -D warnings`
- `cargo fmt --check`

## Success Metrics
- Canonical-equivalent owner DIDs no longer produce false `OwnerNotFound` behavior in M5 owner-scoped APIs.
- Full M5 test suite stays deterministic and green.
