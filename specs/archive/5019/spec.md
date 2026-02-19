# Issue #5019 Spec

- Title: Task: M3 implement blind-index + metadata search APIs with deterministic tests
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M3 requires owner-scoped blind-index search and metadata query APIs so encrypted
messages remain searchable without exposing plaintext. The codebase currently has M0
append-only storage, M1 merkle anchoring, and M2 gateway access controls, but no M3
contract layer that standardizes blind-index normalization, exact-match lookup semantics,
or deterministic metadata filtering behavior.

PRD mapping:
- Section 4.5 (Blind Indexes for Searchable Encryption)
- Section 5.1.1 / 5.2 (messages metadata + search indexes)
- Section 18 scenario 68 (blind index exact-match correctness)

## Acceptance Criteria
- AC-1: Blind-index computation normalizes values deterministically
  (ASCII lowercase + collapsed whitespace) and derives owner-scoped exact-match tokens.
- AC-2: Blind-index search API returns only exact matches and is fail-closed for unsupported
  modes (substring/range/empty inputs).
- AC-3: Metadata search API supports deterministic filtering by sender DID, recipient DID,
  session ID, escrow ID, message type, and bounded created-at range, returning stable ordering.
- AC-4: Search results are owner-scoped and deterministic across insertion order.
- AC-5: Shell/workflow/python LOC remains unchanged for this issue (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M3 module in `kamn-core` for blind-index and metadata search contracts.
- Deterministic in-memory index registration + lookup interfaces.
- Conformance tests for exact-match blind-index correctness and metadata filter determinism.
- Public API exports for follow-on M4+ integration.

Out of scope:
- Live PostgreSQL integration, SQL migrations, and HTTP endpoint wiring.
- Full-text and vector similarity search (M5 scope).
- Dependency additions or wire/protocol changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Compute blind index for semantically identical values with case/spacing differences | Same normalized blind index token |
| C-02 | AC-1/AC-4 | Unit | Compute blind index for same value under different owner salts | Distinct blind index tokens |
| C-03 | AC-2/AC-4 | Conformance | Register records and query by blind-index value | Exact matches returned; non-matching/cross-owner records excluded |
| C-04 | AC-3 | Conformance | Metadata query with sender/session/type/time filters | Deterministic ordered result set that honors all filters |
| C-05 | AC-2 | Regression | Invalid blind-index search mode and empty query inputs | Typed fail-closed errors |
| C-06 | AC-5 | Regression | Inspect issue diff for shell/workflow/python/template files | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m3_blind_index_search`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` conformance tests.
- M3 search contracts are exported via `kamn_core` for M4+ integration.
- Shell-to-Rust posture improves/neutral with zero shell LOC increase.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m3_blind_index_search` failed before implementation with unresolved `DataLayerM3*` symbols.
- GREEN: `cargo test -p kamn-core --test data_layer_m3_blind_index_search` passed (`5 passed, 0 failed`).
- Regression:
  - `cargo fmt --check` passed.
  - `cargo clippy -p kamn-core -- -D warnings` passed.
  - `cargo test -p kamn-core` passed.
- Shell-surface marker:
  - `shell_loc_delta_actual: 0`
  - `rust_loc_delta_actual: +724`
  - `shell_to_rust_ratio_delta_actual: improved`
