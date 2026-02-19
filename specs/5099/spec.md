# Issue #5099 Spec

- Title: Task: integrate M3 blind-index output with content retrieval contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M3 blind-index search returns metadata records but does not expose a canonical contract for generating `ContentRetrievalRequest` inputs. This leaves encrypted search and retrieval integration to ad-hoc caller logic and increases risk of inconsistent authorization/request construction.

## Acceptance Criteria
- AC-1: M3 exposes additive projection contracts that convert blind-index search results into validated `ContentRetrievalRequest` entries.
- AC-2: Projection fails closed when any result message ID has no CID mapping.
- AC-3: Projection fails closed with deterministic taxonomy when retrieval request construction is invalid (for example invalid requester DID).
- AC-4: Existing M3 blind-index and metadata query behavior remains deterministic and backward compatible.
- AC-5: Shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/tests/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5099/{spec.md,plan.md,tasks.md}`

Out of scope:
- Wire/protocol changes.
- New dependencies.
- Runtime transport integration.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Valid blind-index query + complete message->CID map | Deterministic ordered retrieval projection results |
| C-02 | AC-2 | Regression | Blind-index results where one message ID is absent in CID map | Fail-closed deterministic missing-CID error |
| C-03 | AC-3 | Regression | Projection request with invalid requester DID | Fail-closed deterministic invalid-retrieval-request error |
| C-04 | AC-4 | Conformance | Existing M3 conformance corpus (`spec_c01..spec_c08`) | Remains green with stable ordering/reason markers |
| C-05 | AC-5 | Regression | Shell guardrail checks | Zero shell-surface growth and guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m3_blind_index_search`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5099.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5099.json`

## Success Metrics
- M3 provides a typed deterministic contract bridge to content retrieval request construction.
- Fail-closed error paths cover missing CID mappings and invalid retrieval-request construction.
- Shell governance posture remains improved/neutral with zero shell delta.
