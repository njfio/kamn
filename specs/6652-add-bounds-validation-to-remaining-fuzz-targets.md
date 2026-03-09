# 6652 Add Bounds Validation To Remaining Fuzz Targets

## Objective

Add explicit overall input bounds to the remaining unbounded fuzz target and ratchet the fuzz contracts/docs so every current `fuzz/fuzz_targets/*.rs` harness documents or enforces bounded input before deep parsing.

## Inputs/Outputs

- Inputs:
  - Current fuzz targets under `fuzz/fuzz_targets/`
  - Existing cargo-fuzz contract coverage in `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`
  - Fuzz strategy docs in `docs/planning/fuzz_harness_budget_policy.md` and `docs/testing/invariant-and-fuzz-strategy.md`
- Outputs:
  - Bounded overall input handling for the remaining unbounded target
  - Regression/contract coverage that fails closed if a fuzz target omits explicit bounds markers
  - Updated docs describing the bound and why it exists

## Boundaries/Non-goals

- Do not add new fuzz targets in this issue
- Do not redesign parser logic behind the fuzz targets
- Do not widen CI fuzz runtime budgets
- Do not rewrite targets that already have explicit bounds unless needed for contract parity

## Failure Modes

- A fuzz target still feeds effectively unbounded input into deep parser/build logic
- Docs claim bounds coverage that the harness source does not implement
- Contract tests miss a target and allow a future unbounded harness regression
- Added bounds change the target corpus semantics without documenting the cap

## Acceptance Criteria

- [x] Each remaining fuzz target validates input size/shape before deep processing
- [x] Bounds are documented and justified
- [x] Regression tests or harness checks cover the bound behavior
- [x] Fuzz targets continue to run under existing local/CI fuzz workflows
- [x] No remaining target accepts effectively unbounded pathological input without an explicit reason

## Files To Touch

- `specs/6652-add-bounds-validation-to-remaining-fuzz-targets.md`
- `fuzz/fuzz_targets/message_envelope_parser.rs`
- `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`
- `crates/kamn-core/tests/invariant_and_fuzz_strategy_docs.rs`
- `docs/testing/invariant-and-fuzz-strategy.md`
- `docs/ci/strategy.md`

## Error Semantics

- Bounds validation must truncate or reject oversized fuzz input deterministically before deeper parsing/build steps
- Contract tests must fail closed when a target lacks the required bound marker or documented cap
- The harness should preserve panic-free fuzz behavior and deterministic local replay expectations

## Test Plan

- Run `cargo test -p kamn-core --test cargo_fuzz_target_contract -- --nocapture`
- Run `cargo test -p kamn-core --test message_envelope_fuzz_smoke -- --nocapture`
- Run `cargo test -p kamn-core --test invariant_and_fuzz_strategy_docs -- --nocapture`

## Notes / Deviations

- Current repo inspection shows explicit bounds already present in:
  - `did_parser.rs`
  - `signature_profile_parser.rs`
  - `kolme_api_codec_parser.rs`
  - `kolme_flat_json_parser.rs`
  - `kolme_flat_json_policy_parser.rs`
- `message_envelope_parser.rs` is the remaining gap because it bounds individual fields and entry counts but not the overall input slice before envelope construction.

## Refactor Evidence

- The touched fuzz target remains below the file-size limit at `150` LOC.
- Every touched function in `fuzz/fuzz_targets/message_envelope_parser.rs` is within the 25 LOC function limit after extracting:
  - `build_metadata`
  - `build_header`
  - `build_body`
  - `build_attachment`
  - `build_proof`

## Integration Evidence

- `cargo test -p kamn-core --test cargo_fuzz_target_contract -- --nocapture`
  - passed
- `cargo test -p kamn-core --test message_envelope_fuzz_smoke -- --nocapture`
  - passed
- `cargo test -p kamn-core --test invariant_and_fuzz_strategy_docs -- --nocapture`
  - passed

## Deviations

- The issue body described “several remaining targets,” but current repo inspection showed only one real remaining unbounded target:
  - `fuzz/fuzz_targets/message_envelope_parser.rs`
- No other fuzz harnesses were modified because the other current targets already apply explicit deterministic truncation or equivalent input-shape bounds.
