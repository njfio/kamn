# Issue #5021 Spec

- Title: Task: M5 deliver pgvector embeddings pipeline and semantic query endpoints
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M5 defines a vector intelligence layer that supports encrypted embedding
storage, owner-scoped semantic retrieval, and behavioral anomaly scoring.
Current codebase has M0-M4 contract modules but no M5 contract surface that
models embedding registration, deterministic semantic ranking behavior, and
agent centroid-based anomaly evaluation.

PRD mapping:
- Section 6.2 (embedding pipeline and privacy handling)
- Section 6.3 (`message_embeddings` schema fields and controls)
- Section 6.4 (semantic search modes and owner scoping)
- Section 6.5 (centroid-distance anomaly detection)
- Milestone table M5 deliverables (pgvector integration + anomaly scoring)

## Acceptance Criteria
- AC-1: Embedding registry contract accepts owner-scoped encrypted embedding
  inserts and enforces append-only deterministic record-hash chaining with fail-closed errors.
- AC-2: Semantic query contract returns deterministic top-k ranking for
  owner-scoped embeddings using cosine-similarity semantics.
- AC-3: Privacy mode contract enforces owner-side encrypted vs server-side
  plaintext opt-in behavior with explicit denial markers when query prerequisites are missing.
- AC-4: Anomaly scoring contract computes per-agent centroid distance and
  threshold-triggered anomaly decisions with stable reason codes.
- AC-5: Shell/workflow/python/template LOC remains unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M5 contract module in `kamn-core` for embedding registration, semantic query,
  and anomaly scoring contracts.
- Conformance tests for deterministic ranking, owner scope isolation, privacy-mode gating,
  and anomaly threshold classification.
- Public export wiring for downstream M6+ integration lanes.

Out of scope:
- PostgreSQL extension DDL/migrations and runtime SQL execution.
- External embedding model SDK integration or network calls.
- New dependencies, wire/protocol changes, or shell/python workflow additions.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Register embeddings for one owner with deterministic sequence | Records append with stable hash-chain and deterministic `record_hash` markers |
| C-02 | AC-1/AC-2 | Unit | Attempt duplicate embedding id registration | Fail-closed duplicate error is returned |
| C-03 | AC-2 | Conformance | Query top-k semantic neighbors within one owner scope | Deterministic ranking and owner isolation are preserved |
| C-04 | AC-3 | Regression | Query without plaintext vectors under owner-side encrypted mode | Query denied with stable privacy-mode reason marker |
| C-05 | AC-4 | Conformance | Evaluate anomaly score from centroid distance over agent window | Threshold crossing emits anomaly=true with stable reason marker |
| C-06 | AC-5 | Regression | Inspect issue diff for shell/python/workflow/template files | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m5_vector_integration`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` conformance tests.
- M5 contracts are exported via `kamn_core` for downstream task wiring.
- Shell-to-Rust ratio direction is improved/neutral through Rust-only changes.

## Verification Evidence
- RED: `cargo test -p kamn-core --test data_layer_m5_vector_integration` failed before implementation with unresolved `DataLayerM5*` symbols.
- GREEN: `cargo test -p kamn-core --test data_layer_m5_vector_integration` passed after module implementation and exports.
- REGRESSION: `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, and `cargo test -p kamn-core` pass.

## AC Verification
| AC | Result | Tests |
|---|---|---|
| AC-1 | ✅ | `spec_c01_embedding_registry_append_is_deterministic_and_hash_chained`; `spec_c02_duplicate_embedding_id_is_rejected_fail_closed` |
| AC-2 | ✅ | `spec_c03_semantic_query_is_owner_scoped_and_ranked_deterministically` |
| AC-3 | ✅ | `spec_c04_owner_side_encrypted_mode_rejects_server_side_semantic_query` |
| AC-4 | ✅ | `spec_c05_anomaly_threshold_detection_uses_centroid_distance_rules` |
| AC-5 | ✅ | Diff inspection for issue files confirms Rust-only surface |

## Shell Surface Markers
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: +1040
- shell_to_rust_ratio_delta_actual: -0.007495
- shell_surface_ratio_target_status: improved
