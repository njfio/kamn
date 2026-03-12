# 6907-split-data-layer-m4-escrow-integration

## Objective
Split `crates/kamn-core/src/data_layer_m4_escrow_integration.rs` into bounded concern-based modules while preserving escrow lifecycle transitions, scoped visibility decisions, settlement evidence registry behavior, reconciliation semantics, and existing deterministic validation behavior.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/data_layer_m4_escrow_integration.rs` production source and its downstream consumers in `kamn-core` tests and runtime paths
- Output: a thin root shell delegating to bounded sibling modules for escrow models/errors, transition engine, visibility logic, settlement evidence registry/reconciliation, validation/hash helpers, and tests
- Output: a hard-fail extraction contract enforcing the new module layout and root shell budget

## Boundaries/Non-goals
- Do not change escrow state semantics, reason codes, or reconciliation outcomes
- Do not redesign evidence storage formats or hashing rules beyond extraction seams
- Do not add new escrow features, public API fields, or external dependencies
- Do not weaken or delete existing escrow behavior tests to make the split pass

## Failure modes
- Extraction contract passes while the root file remains oversized or expected modules are missing
- Escrow transition validation or reason-code selection changes during extraction
- Visibility decisions for participants/auditors drift silently
- Settlement evidence registry ordering, hash-chain, or reconciliation semantics change
- Any touched extracted file or function fails the touched-Rust size policy

## Acceptance criteria
- [x] `crates/kamn-core/src/data_layer_m4_escrow_integration.rs` becomes a thin root shell under the active file-size policy
- [x] bounded sibling modules exist for escrow state/transition models, visibility decisions, settlement evidence registry/reconciliation, validation/hash helpers, and tests
- [x] existing escrow lifecycle, visibility, and settlement evidence behavior remains unchanged after the split
- [x] a hard-fail extraction contract exists and passes
- [x] the real `data_layer_m4_escrow_integration` behavior target still passes after the split
- [x] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/data_layer_m4_escrow_integration.rs`
- `crates/kamn-core/src/data_layer_m4_escrow_integration/`
- `crates/kamn-core/tests/data_layer_m4_escrow_integration_module_extraction_contract.rs`
- `specs/6907-split-data-layer-m4-escrow-integration.md`

## Error semantics
- Existing fail-closed escrow and settlement evidence errors remain typed and externally observable
- No new fallbacks, silent coercions, or swallowed validation failures are introduced
- Invalid DIDs, timestamps, hashes, and transition requests remain deterministic and explicit

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing
- Run the extraction contract target
- Run the escrow integration behavior target after extraction
- Run touched-Rust size policy on the final branch

## Final evidence
- `cargo test -p kamn-core --test data_layer_m4_escrow_integration_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test data_layer_m4_escrow_integration -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6907-touched-size-green.json`
- touched-Rust result: `policy_decision=GO`

## Deviations
- None
