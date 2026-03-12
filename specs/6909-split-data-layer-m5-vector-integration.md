# 6909-split-data-layer-m5-vector-integration

## Objective
Split `crates/kamn-core/src/data_layer_m5_vector_integration.rs` into bounded concern-based modules while preserving deterministic M5 vector ingestion, semantic query ranking, anomaly evaluation, retention-due projection, recall-drift behavior, and existing tests.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/data_layer_m5_vector_integration.rs` production source and downstream `kamn-core` tests that exercise M5 vector behavior
- Output: a thin root shell delegating to bounded sibling modules for models/constants, registry operations, semantic query/retrieval flow, anomaly and recall-drift flow, retention helpers, validation/utilities, and tests
- Output: a hard-fail extraction contract enforcing root shell and module layout

## Boundaries/Non-goals
- Do not change vector scoring semantics, anomaly thresholds, or retention calculations
- Do not change public data fields or reason-code values
- Do not add dependencies or alter privacy-mode semantics
- Do not weaken or delete existing M5 behavior tests to make the split pass

## Failure modes
- Root file remains oversized while extraction contract passes
- Semantic query ordering or cosine-similarity behavior changes during extraction
- Anomaly or recall-drift decisions drift silently
- Retention-due projection changes output rows or reason codes
- Any touched extracted file or function fails the touched-Rust size policy

## Acceptance criteria
- [ ] `crates/kamn-core/src/data_layer_m5_vector_integration.rs` becomes a thin root shell under the active file-size policy
- [ ] bounded sibling modules exist for models/constants, registry operations, semantic query/retrieval flow, anomaly/recall-drift flow, retention/projection helpers, and tests
- [ ] existing `data_layer_m5_vector_integration` behavior remains unchanged after the split
- [ ] a hard-fail extraction contract exists and passes
- [ ] the real `data_layer_m5_vector_integration` behavior target still passes after the split
- [ ] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs`
- `crates/kamn-core/src/data_layer_m5_vector_integration/`
- `crates/kamn-core/tests/data_layer_m5_vector_integration_module_extraction_contract.rs`
- `specs/6909-split-data-layer-m5-vector-integration.md`

## Error semantics
- Existing fail-closed M5 validation and registry errors remain typed and externally observable
- No new fallbacks, silent coercions, or swallowed scoring/retention failures are introduced
- Invalid DIDs, vectors, recall baselines, and threshold inputs remain deterministic and explicit

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing
- Run the extraction contract target
- Run the M5 vector integration behavior target after extraction
- Run touched-Rust size policy on the final branch
