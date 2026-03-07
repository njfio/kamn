# 6556 Restore Script-Surface Doc Inventory Parity

## Objective
Restore parity between the checked-in script-surface developer docs and the current `scripts/` filesystem inventory so the existing `kamn-core` docs contract tests pass again on `main`.

## Inputs/Outputs
- Inputs:
  - Current `scripts/` filesystem inventory
  - `docs/developer/script-surface-index.md`
  - `docs/developer/script-surface-reduction-candidates.md`
  - Existing contract tests in `crates/kamn-core/tests/script_surface_index_docs.rs` and `crates/kamn-core/tests/script_surface_reduction_candidates_docs.rs`
- Outputs:
  - Updated docs markers and tables that match the filesystem inventory
  - Passing targeted contract tests for both docs surfaces

## Boundaries/Non-goals
- No new shell or Python scripts
- No workflow or CI logic changes
- No taxonomy changes for script categories
- No behavior changes outside the two developer docs and their contract coverage

## Failure modes
- `script-surface-index` markers remain stale and continue reporting an incorrect Python total
- `script-surface-index` category table remains out of sync with the actual `scripts/ci` counts
- `script-surface-reduction-candidates` candidate table remains out of sync with the actual `scripts/ci` total
- Added assertions point at the wrong canonical counts and create false negatives

## Acceptance criteria
- [ ] `docs/developer/script-surface-index.md` records `748` shell files, `334` Python files, `1082` total files, and `scripts/ci` as `147` shell + `51` Python = `198` total
- [ ] `docs/developer/script-surface-reduction-candidates.md` records `scripts/ci` as `198` total scripts and preserves the correct `19` short-wrapper candidates
- [ ] `cargo test -p kamn-core --test script_surface_index_docs -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test script_surface_reduction_candidates_docs -- --nocapture` passes

## Files to touch
- `specs/6556-restore-script-surface-doc-inventory-parity.md`
- `docs/developer/script-surface-index.md`
- `docs/developer/script-surface-reduction-candidates.md`
- `crates/kamn-core/tests/script_surface_index_docs.rs`
- `crates/kamn-core/tests/script_surface_reduction_candidates_docs.rs`

## Error semantics
- Docs contract failures remain hard-fail `assert_eq!` / `assert!` failures in test code
- No silent normalization or inferred fallback counts are allowed; docs must exactly match filesystem inventory

## Test plan
1. Add targeted red assertions for the canonical `scripts/ci` inventory counts in both docs contract tests.
2. Run `cargo test -p kamn-core --test script_surface_index_docs -- --nocapture` and confirm failure before docs updates.
3. Run `cargo test -p kamn-core --test script_surface_reduction_candidates_docs -- --nocapture` and confirm failure before docs updates.
4. Update the two docs to the current filesystem inventory.
5. Re-run both targeted tests and confirm they pass.
