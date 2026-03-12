# 6903-split-block-pipeline-support

## Objective
Split `crates/kamn-core/src/block_pipeline/block_pipeline_support.rs` into bounded concern-based modules while preserving gossip ingress decoding, transport-fed mempool/canonical candidate ingestion, convergence evidence generation, canonical commit stores, serialization/parsing, and fork-choice behavior.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/block_pipeline/block_pipeline_support.rs` production source plus existing transport-fed pipeline runtime/tests that consume its public surface through `block_pipeline.rs` and downstream crates
- Output: a thin root shell that delegates to bounded sibling modules for the extracted block-pipeline support concerns
- Output: a hard-fail extraction contract enforcing the new module layout

## Boundaries/Non-goals
- Do not change block-pipeline semantics or public API names
- Do not redesign adjacent block-pipeline modules beyond the seams needed for this extraction
- Do not add new dependencies, persistence backends, or runtime features
- Do not weaken or delete existing block-pipeline tests to make the split pass

## Failure modes
- Extraction contract passes while `block_pipeline_support.rs` remains oversized or expected modules are missing
- Public re-export surface drifts and breaks downstream compilation silently
- Gossip ingress decoding, transport mempool/candidate feed behavior, convergence evidence generation, canonical commit store behavior, serialization/parsing, or fork-choice behavior changes during extraction
- Any touched extracted file exceeds the touched-Rust size policy
- Final branch still fails touched-Rust size policy

## Acceptance criteria
- [ ] `crates/kamn-core/src/block_pipeline/block_pipeline_support.rs` becomes a thin root shell under the active file-size policy
- [ ] bounded sibling modules exist for gossip ingress, transport feeds, convergence evidence, canonical commit stores, serialization/parsing, and fork-choice support
- [ ] the block-pipeline public surface remains wired and downstream compilation continues to succeed
- [ ] a hard-fail extraction contract exists and passes
- [ ] real block-pipeline coverage for the touched domains still passes after the split
- [ ] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/block_pipeline/block_pipeline_support.rs`
- `crates/kamn-core/src/block_pipeline/block_pipeline_support/`
- `crates/kamn-core/tests/block_pipeline_support_module_extraction_contract.rs`
- `specs/6903-split-block-pipeline-support.md`

## Error semantics
- No new fallbacks or swallowed failures in gossip ingress, transport feed, persistence, or fork-choice paths
- Existing typed error behavior remains fail-closed and externally observable through current return types
- Store decode/parse failures remain deterministic and explicit

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing
- Run the extraction contract target
- Run the transport-fed block-pipeline coverage that exercises the touched domains after the split
- Run touched-Rust size policy on the final branch
