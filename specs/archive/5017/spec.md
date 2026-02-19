# Issue #5017 Spec

- Title: Task: M1 implement merkle batching, Kolme anchoring worker, and proof APIs
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
PRD M1 requires a deterministic trust-anchor layer that assembles message content hashes into merkle
batches, generates and verifies inclusion proofs, and anchors batch roots through the Kolme runtime
commit surface with idempotent retry semantics. The current codebase has M0 append-only + hash-chain
contracts but does not yet provide an M1 merkle batch/proof/anchoring API that follow-on milestones
can consume directly.

PRD mapping:
- Section 4.3 (Content Hash and Merkle Anchoring)
- Section 5.1.2 (`merkle_batches` anchoring record model)
- Section 15.1 (`Merkle Anchoring Worker`)
- Section 17/18 scenarios 64-65 (proof verification + hash tamper detection)

## Acceptance Criteria
- AC-1: A deterministic merkle batch API exists for ordered content hashes and emits stable
  `merkle_root`, `message_count`, `first_message_id`, `last_message_id`, and tree height outputs
  independent of caller input ordering.
- AC-2: Inclusion proof APIs can generate and verify per-message proofs against the batch root;
  tampered proofs, tampered leaf hashes, and unknown message lookups fail closed.
- AC-3: A Kolme anchoring worker builds deterministic runtime-commit submissions for merkle roots,
  preserving idempotency and classifying outcomes across `Submitted`, `Duplicate`, and `Rejected`.
- AC-4: Invalid batch inputs (empty set, duplicate/non-contiguous leaf indexes, empty fields) are
  rejected with typed errors before any anchoring attempt.
- AC-5: Shell/workflow/python LOC remains unchanged for this issue (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- New Rust M1 module in `kamn-core` for merkle batching, proof generation/verification, and anchoring worker contracts.
- Deterministic in-memory + Kolme-runtime-commit backed anchoring integration surfaces.
- Spec-derived unit/functional/conformance/regression tests for C-01..C-05.
- Public re-exports from `kamn_core` for downstream M2+ integration.

Out of scope:
- PostgreSQL table migrations, storage workers, and background scheduling.
- External networked Kolme integration tests requiring live infrastructure.
- Dependency additions or wire-format changes requiring ADR/approval.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Same leaf set provided in different vector order with explicit `leaf_index` | Identical `merkle_root` and batch metadata |
| C-02 | AC-2 | Functional | Generate proof for one message and verify against emitted root | Verification succeeds and returns stable proof contract |
| C-03 | AC-2 | Regression | Tamper proof sibling hash / leaf content hash / request unknown message id | Verification or lookup fails with typed fail-closed error |
| C-04 | AC-3 | Integration | Anchor same batch twice through in-memory Kolme client | First outcome `Submitted`, second outcome `Duplicate` with stable idempotency key |
| C-05 | AC-4 | Unit | Build batch from empty/invalid leaves or non-contiguous indexes | Constructor rejects with typed validation error |
| C-06 | AC-5 | Regression | Inspect diff for shell/python/workflow/template touches | Net shell-surface delta remains zero |

## Test Mapping
- `cargo test -p kamn-core data_layer_m1_merkle_anchoring`
- `cargo test -p kamn-core spec_c0`
- `cargo test -p kamn-core`
- Shell governance scripts are not required because shell/workflow surfaces are unchanged.

## Success Metrics
- All ACs map to passing `spec_c0x_*` tests under `kamn-core`.
- M1 public contracts are exported and consumable by follow-on milestone code.
- Shell-to-Rust guardrail posture is improved/neutral with zero new shell LOC.
