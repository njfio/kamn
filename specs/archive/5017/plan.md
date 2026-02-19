# Issue #5017 Plan

- Issue: #5017
- Status: Reviewed

## Approach
1. Add red tests for C-01..C-05 in a dedicated `kamn-core` M1 contract file covering:
   - deterministic root stability with reordered inputs,
   - proof generation + verification,
   - fail-closed tamper detection,
   - Kolme anchoring worker idempotency and retry classification,
   - invalid-input guardrails.
2. Implement `data_layer_m1` with:
   - deterministic leaf canonicalization and merkle tree assembly,
   - inclusion proof builder and verifier,
   - typed error taxonomy and stable reason-code helpers,
   - Kolme anchoring worker contract using `KolmeRuntimeCommitClient`.
3. Re-export public M1 types/functions from `crates/kamn-core/src/lib.rs`.
4. Run scoped and crate-wide regression tests; update spec/tasks lifecycle markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m1.rs` (new)
- `crates/kamn-core/src/lib.rs` (module declaration + re-exports)
- `crates/kamn-core/tests/data_layer_m1_merkle_anchoring.rs` (new)
- `specs/5017/spec.md`
- `specs/5017/plan.md`
- `specs/5017/tasks.md`

## Risks and Mitigations
- Risk level: high
- Risks:
  - Merkle canonicalization drift causing root mismatch across call-sites.
  - Incorrect sibling ordering in proof verification producing false positives.
  - Anchoring retry/idempotency mismatch across duplicate submissions.
- Mitigations:
  - Canonicalize leaves by explicit `leaf_index` and enforce contiguous indexes.
  - Encode proof steps with explicit sibling side (`left`/`right`) and validate every step.
  - Keep deterministic idempotency key derivation tied to stable batch payload fields.
  - Add regression tests for tampered proof data and duplicate anchoring retries.

## Interface Contract
- Additive-only public API under `kamn_core::data_layer_m1::*`.
- No new external dependencies.
- No protocol/wire-format changes; Kolme integration remains via existing `KolmeRuntimeCommitClient`.

## ADR
- Not required for this bounded additive implementation (no dependency or architecture pivot).
