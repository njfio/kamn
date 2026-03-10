# 6834-split-production-expect-surface-policy-support

## Objective
Reduce `crates/kamn-core/tests/support/production_expect_surface_policy_support.rs` from a 513 LOC support monolith to a thin root shell plus bounded helper modules without changing production-expect policy behavior or weakening the tests that depend on this support surface.

## Inputs/Outputs
- Input: existing production-expect surface policy support helpers in `crates/kamn-core/tests/support/production_expect_surface_policy_support.rs`
- Output: bounded sibling support modules and a root shell that wires them together
- Output: extraction contract coverage enforcing the root shell budget and module layout

## Boundaries/Non-goals
- No production code changes
- No new dependencies
- No changes to public API or panic/error semantics exposed to dependent tests
- No weakening of production-expect policy assertions or taxonomy markers

## Failure Modes
- Root shell remains above the staged 180 LOC cap
- Any touched extracted file exceeds 200 LOC
- Dependent tests lose access to required support symbols after extraction
- Baseline/threshold loading or census behavior changes during the split
- Touched-Rust size policy remains `NO-GO`

## Acceptance Criteria
- [ ] `crates/kamn-core/tests/support/production_expect_surface_policy_support.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] The root shell wires bounded sibling support modules for fixture parsing, baseline/threshold loading, source census, test-only path classification, and raw scanning helpers
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] Existing tests that depend on this support surface still pass without assertion drift
- [ ] A new extraction contract for the root shell passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json <path>` returns `policy_decision=GO`

## Files To Touch
- `specs/6834-split-production-expect-surface-policy-support.md`
- `crates/kamn-core/tests/support/production_expect_surface_policy_support.rs`
- `crates/kamn-core/tests/support/production_expect_surface_policy_support/`
- `crates/kamn-core/tests/production_expect_surface_policy_support_extraction_contract.rs`

## Error Semantics
- Preserve existing fail-closed panic behavior and reason-code strings unchanged
- Extraction contract failures must fail with explicit missing-file, missing-marker, or root-budget assertions
- No silent fallbacks or marker drift

## Test Plan
1. Add an extraction contract that fails while `production_expect_surface_policy_support.rs` remains monolithic.
2. Split the root into bounded modules by concern:
   - fixture parsing helpers
   - baseline and threshold loading
   - tracked source census helpers
   - test-only source path classification
   - raw token scanning helpers
3. Run the extraction contract.
4. Run the dependent production-expect policy tests.
5. Run the touched-Rust size ratchet and require `policy_decision=GO`.

## Phase 6 Evidence
- Root shell wiring verified through `cargo test -p kamn-core --test production_expect_surface_policy -- --nocapture`
- Extraction contract verified through `cargo test -p kamn-core --test production_expect_surface_policy_support_extraction_contract -- --nocapture`
- Touched-Rust policy verified through `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6834-touched-size.json`
- Result: `policy_decision=GO`
