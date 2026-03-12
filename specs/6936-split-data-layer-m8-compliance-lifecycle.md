# 6936-split-data-layer-m8-compliance-lifecycle

## Objective
Split `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs` into bounded, concern-based modules while preserving retention-window mapping, owner-scope authorization, legal-hold precedence, crypto-shred transitions, and existing typed error behavior.

## Inputs/Outputs
- Inputs:
  - owner-scoped message lifecycle records
  - legal-hold mutation requests
  - crypto-shred mutation requests
  - owner-scope retention queries
  - retention class interop conversions
- Outputs:
  - unchanged M8 compliance lifecycle behavior
  - a thin root shell in `data_layer_m8_compliance_lifecycle.rs`
  - bounded sibling modules for retention policy, models, registry/store logic, mutation/query surface, and tests
  - a hard-fail extraction contract for the root shell and module layout

## Boundaries/Non-goals
- No changes to retention-window semantics, shred markers, or stable reason codes
- No changes to owner-scope authorization policy
- No new dependencies
- No unrelated data-layer refactors outside the M8 compliance lifecycle surface

## Failure modes
- invalid owner DID scope remains fail-closed
- duplicate message registration remains fail-closed
- unauthorized owner-scope query or mutation remains fail-closed
- legal-hold precedence remains fail-closed
- invalid shred timing or already-shredded transitions remain fail-closed
- extraction contract fails if the root shell or module layout regress

## Acceptance criteria
- [ ] `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs` becomes a thin root shell under the active file-size budget
- [ ] bounded modules separate retention policy, models, registry/store logic, lifecycle mutations/query surface, and tests
- [ ] a hard-fail extraction contract enforces the root shell and module layout
- [ ] existing M8 compliance lifecycle tests remain green without semantic drift
- [ ] touched-Rust size policy returns `policy_decision=GO`
- [ ] final spec records test evidence and any deviations

## Files to touch
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs`
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle/`
- `crates/kamn-core/tests/data_layer_m8_compliance_lifecycle_module_extraction_contract.rs`
- `specs/6936-split-data-layer-m8-compliance-lifecycle.md`

## Error semantics
- Preserve existing typed error behavior and stable reason markers
- Preserve fail-closed validation for owner scope, retention windows, legal hold, and crypto-shred transitions
- Do not introduce silent fallback or relaxed authorization behavior

## Test plan
- Add a red extraction contract that fails while `data_layer_m8_compliance_lifecycle.rs` remains monolithic
- Run the extraction contract green once the split is in place
- Run the real M8 compliance lifecycle tests after extraction
- Run touched-Rust size policy against the staged write set
