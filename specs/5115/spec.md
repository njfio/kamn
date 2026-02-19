# Issue #5115 Spec

- Title: Task: canonicalize M6 owner DID scope using KamnDid parser
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M6 graph integration validates owner DIDs using a local string checker but compares requester and owner scope using raw strings. Equivalent owner DIDs with whitespace differences can pass validation and still fail scope authorization or owner-map lookup, causing false denials.

## Acceptance Criteria
- AC-1: M6 owner DID parsing uses canonical `KamnDid::parse` paths in registration, authorization, and projection APIs.
- AC-2: Owner scope comparisons in M6 are performed on canonical DID strings rather than raw input strings.
- AC-3: Owner lookup paths (`nodes_for_owner`, `edges_for_owner`, projection export APIs) resolve using canonical owner DID keys.
- AC-4: M6 deterministic behavior remains stable and regression tests pass.

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m6_graph_integration.rs`
- `crates/kamn-core/tests/data_layer_m6_graph_integration.rs`
- `specs/5115/{spec.md,plan.md,tasks.md}`

Out of scope:
- M6 graph schema changes
- New dependencies
- Non-M6 module refactors

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Register/query/export with owner DIDs containing surrounding whitespace | Canonical `KamnDid` parsing path is used and operations succeed/fail deterministically |
| C-02 | AC-2 | Conformance | Scoped export where requester has whitespace-wrapped equivalent owner DID | Request is authorized (no false owner-scope denial) |
| C-03 | AC-3 | Conformance | Owner lookup APIs called with whitespace-wrapped equivalent DID | Existing owner graph entries are returned via canonical lookup |
| C-04 | AC-4 | Regression | Existing M6 conformance tests + new canonicalization tests | All pass with deterministic ordering and reason codes preserved |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m6_graph_integration -- spec_c08_scoped_projection_accepts_canonical_equivalent_owner_dids`
- `cargo test -p kamn-core --test data_layer_m6_graph_integration -- spec_c09_owner_lookup_uses_canonical_kamn_did_keys`
- `cargo test -p kamn-core --test data_layer_m6_graph_integration`
- `cargo clippy -p kamn-core -- -D warnings`
- `cargo fmt --check`

## Success Metrics
- Equivalent owner DID strings no longer produce false scope-denied outcomes in M6.
- No regressions in existing M6 conformance suite.
