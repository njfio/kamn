# Plan: Issue #5840 - cfg(test) Parsing Hardening for `expect()` Inventory

- Issue: #5840
- Spec: `specs/5840/spec.md`
- Status: Implemented

## Approach
1. Add a RED regression fixture in the checker shell harness reproducing cfg(test)-item leakage with
   brace-heavy strings.
2. Replace naive line-level brace counting with scanner logic that ignores braces inside strings,
   chars, and comments while skipping cfg(test) items.
3. Mirror equivalent skip logic in `review_r53_docs_contract.rs` inventory helper to preserve
   checker/docs-contract parity.
4. Align R55 marker formula string to the implemented cfg(test)-aware semantics.
5. Run targeted lanes and repair any drift.

## Affected Modules
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `crates/kamn-core/tests/review_r53_docs_contract.rs`
- `docs/review/gaps-and-issues-r55.md`
- `specs/5840/{spec,plan,tasks}.md`

## Risks and Mitigations
- Risk: Scanner change could over-skip and hide real production violations.
  - Mitigation: keep existing violation fixtures and add explicit leakage regression fixture.
- Risk: Rust/Python parser behavior diverges.
  - Mitigation: implement equivalent rules and assert docs-contract inventory invariants.
- Risk: Marker text/value drift in R55 contract.
  - Mitigation: update marker text and run review docs-contract lane.

## Interfaces / Contracts
- `check_no_production_expect.py` remains CLI-compatible.
- Reason code taxonomy remains unchanged.
- R55 marker schema key names remain unchanged; formula value text is corrected.

## ADR
- Not required: no architecture/protocol/dependency changes.
