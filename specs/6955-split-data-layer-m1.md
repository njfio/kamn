# 6955-split-data-layer-m1

## Objective
Reduce `crates/kamn-core/src/data_layer_m1.rs` from its monolithic form into a bounded root shell plus concern-based submodules without changing deterministic M1 merkle, proof-verification, or Kolme anchoring behavior.

## Inputs/Outputs
- Inputs:
  - Existing M1 merkle batch assembly, inclusion-proof verification, anchoring worker, failure-matrix evaluation, and inline tests in `crates/kamn-core/src/data_layer_m1.rs`
  - Existing public callers through `kamn-core`
- Outputs:
  - Thin root shell at `crates/kamn-core/src/data_layer_m1.rs`
  - Extracted module tree under `crates/kamn-core/src/data_layer_m1/`
  - Hard-fail extraction contract covering root shell budget, expected module markers, and moved-inline markers

## Boundaries/Non-goals
- No behavior changes to M1 hashing, proof, anchoring, or failure-matrix semantics
- No public API redesign beyond re-export wiring required by the split
- No unrelated cleanup outside the M1 source, extraction contract, and this issue spec

## Failure modes
- Missing extracted module files
- Root shell still contains moved types, helpers, or tests inline
- Root shell exceeds staged line budget
- Split breaks public imports or M1 worker/proof behavior
- Split introduces touched-Rust size regressions in new files or functions

## Acceptance criteria
- [x] `crates/kamn-core/src/data_layer_m1.rs` is reduced to a bounded root shell
- [x] Extracted modules under `crates/kamn-core/src/data_layer_m1/` stay within the active size policy
- [x] Existing M1 behavior remains green under real tests/checks
- [x] `crates/kamn-core/tests/data_layer_m1_module_extraction_contract.rs` hard-fails on layout regressions
- [x] Touched-Rust size policy returns `policy_decision=GO`

## Files to touch
- `crates/kamn-core/src/data_layer_m1.rs`
- `crates/kamn-core/src/data_layer_m1/*.rs`
- `crates/kamn-core/tests/data_layer_m1_module_extraction_contract.rs`
- `specs/6955-split-data-layer-m1.md`

## Error semantics
- Preserve existing `Result`-based and typed-error behavior
- Preserve current `DataLayerM1Error` taxonomy and message context
- No silent fallback or swallowed verification/anchoring failures

## Test plan
1. Add a red extraction contract that fails while the root file remains monolithic
2. Extract the module tree and re-run the extraction contract to green
3. Run issue-local behavior checks for M1 proof/anchoring compilation and tests
4. Run touched-Rust size policy against the issue write set

## Verification
- `cargo test -p kamn-core --test data_layer_m1_module_extraction_contract -- --nocapture`
- `cargo check -p kamn-core --lib`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-active-20260313-093857 --base-ref origin/main --output-json /tmp/6955-touched-size-final.json`

## Deviations
- `cargo test -p kamn-core data_layer_m1::tests:: --lib -- --nocapture` is still blocked on the unrelated current-main parse error in `crates/kamn-core/src/data_layer_m7_timeseries_telemetry/tests.rs:84`.
- The issue was still integrated because the extraction contract, full `kamn-core` lib check, and touched-Rust policy all passed for the M1 split itself.
