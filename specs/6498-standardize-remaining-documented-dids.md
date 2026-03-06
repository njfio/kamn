## Objective
Standardize the remaining documented example call sites that still use the divergent DID shape
`did:kamn:...` so the repository’s published examples match the canonical parser surface
`kamn:did:{role}:{id}`.

## Inputs/Outputs
- Inputs:
  - `docs/foundation/reputation-signal-routing.md`
  - `docs/foundation/audit-export-interfaces.md`
  - `docs/foundation/reputation-state-model.md`
  - `docs/foundation/release-gonogo-checklist.md`
  - `docs/foundation/data-classification-tagging.md`
  - `docs/architecture/did-format-standardization.md`
  - existing doc contract tests in `crates/kamn-core/tests/*` and
    `crates/kamn-types/tests/identity_boundary_contract.rs`
- Outputs:
  - canonical `kamn:did:...` example values in the remaining docs
  - doc contract assertions that pin the canonical examples
  - DID inventory text and identity boundary contract updated to the zero-active-consumer state

## Boundaries/Non-goals
- Do not change parser, runtime, CLI, SDK, or script behavior.
- Do not introduce compatibility shims or public wire-format changes.
- Do not modify shell/python/workflow surfaces.

## Failure modes
- Any of the remaining docs still show `did:kamn:...` example values.
- The DID inventory still claims active divergent doc consumers after the docs are cleaned.
- Doc contract tests pass only because assertions were weakened instead of pinning canonical values.
- Search results still show non-intentional `did:kamn:` usage outside the DID inventory document.

## Acceptance criteria
- [ ] `docs/foundation/reputation-signal-routing.md` uses canonical `kamn:did:...` examples.
- [ ] `docs/foundation/audit-export-interfaces.md` uses canonical `kamn:did:...` examples.
- [ ] `docs/foundation/reputation-state-model.md` uses canonical `kamn:did:...` examples.
- [ ] `docs/foundation/release-gonogo-checklist.md` uses canonical `kamn:did:...` examples.
- [ ] `docs/foundation/data-classification-tagging.md` uses canonical `kamn:did:...` examples.
- [ ] existing doc contract tests pin the canonical examples in each affected document.
- [ ] `docs/architecture/did-format-standardization.md` records that no active divergent doc
      consumers remain outside the intentional divergence-description section.
- [ ] `crates/kamn-types/tests/identity_boundary_contract.rs` matches the zero-active-consumer
      inventory state.
- [ ] Focused doc contract commands pass locally.

## Files to touch
- `docs/foundation/reputation-signal-routing.md`
- `docs/foundation/audit-export-interfaces.md`
- `docs/foundation/reputation-state-model.md`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/foundation/data-classification-tagging.md`
- `docs/architecture/did-format-standardization.md`
- `crates/kamn-core/tests/docs_contract_wave4_harness.rs`
- `crates/kamn-core/tests/reputation_state_model_docs.rs`
- `crates/kamn-core/tests/audit_export_interfaces_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/data_classification_tagging_docs.rs`
- `crates/kamn-types/tests/identity_boundary_contract.rs`
- `specs/6498-standardize-remaining-documented-dids.md`

## Error semantics
- No runtime or parser error semantics change in this issue.
- This issue only changes published example values, inventory wording, and documentation contracts.

## Test plan
- Red:
  - add/strengthen doc contract assertions so they require canonical `kamn:did:...` example
    values and a zero-active-consumer inventory state
  - run the focused doc contract commands and confirm they fail before the docs are updated
- Green:
  - `cargo test -p kamn-core --test docs_contract_wave4_harness reputation_signal_routing_docs::regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test reputation_state_model_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test audit_export_interfaces_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test data_classification_tagging_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture`
- Refactor:
  - rerun the focused commands after inventory and doc cleanup
  - run `rg -n "did:kamn:" docs crates` to verify only intentional inventory references remain
