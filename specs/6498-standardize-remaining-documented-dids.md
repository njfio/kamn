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
- [x] `docs/foundation/reputation-signal-routing.md` uses canonical `kamn:did:...` examples.
- [x] `docs/foundation/audit-export-interfaces.md` uses canonical `kamn:did:...` examples.
- [x] `docs/foundation/reputation-state-model.md` uses canonical `kamn:did:...` examples.
- [x] `docs/foundation/release-gonogo-checklist.md` uses canonical `kamn:did:...` examples.
- [x] `docs/foundation/data-classification-tagging.md` uses canonical `kamn:did:...` examples.
- [x] existing doc contract tests pin the canonical examples in each affected document.
- [x] `docs/architecture/did-format-standardization.md` records that no active divergent doc
      consumers remain outside the intentional divergence-description section.
- [x] `crates/kamn-types/tests/identity_boundary_contract.rs` matches the zero-active-consumer
      inventory state.
- [x] Focused doc contract commands pass locally.

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
  - run `rg -n "did:kamn:" docs` to verify only intentional inventory references remain

## Deviations
- The existing repository already had doc contract tests covering every affected document, so this
  issue extended those contract surfaces instead of introducing new standalone doc test files.
- The DID inventory document now carries machine-readable zero-consumer and inventory-only markers
  so the remaining deprecated token references are explicitly scoped to documentation inventory
  text, not active example call sites.

## Execution Evidence
- Red:
  - `cargo test -p kamn-core --test docs_contract_wave4_harness reputation_signal_routing_docs::regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test reputation_state_model_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test audit_export_interfaces_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test data_classification_tagging_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture`
- Green / Refactor / Integration:
  - `cargo test -p kamn-core --test docs_contract_wave4_harness reputation_signal_routing_docs::regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test reputation_state_model_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test audit_export_interfaces_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-core --test data_classification_tagging_docs regression_requires_canonical_did_examples -- --exact --nocapture`
  - `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture`
  - `rg -n "did:kamn:" docs | sed -n '1,200p'`
