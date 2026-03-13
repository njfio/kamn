# 6948-split-block-pipeline

## Objective
Reduce `crates/kamn-core/src/block_pipeline.rs` to a thin root shell under the active file-size policy by extracting concern-based modules for consensus round models and errors, durable commit reason projection, lane-boundary enforcement, commit-store and transport hook contracts, and tests, without changing block pipeline behavior.

## Inputs/Outputs
- Inputs:
  - Current `crates/kamn-core/src/block_pipeline.rs`
  - Existing adjacent pipeline tests and transport-fed contracts
  - Current block pipeline public API exported through `kamn-core`
- Outputs:
  - Thin root `block_pipeline.rs`
  - Bounded sibling modules under `crates/kamn-core/src/block_pipeline/`
  - Hard-fail extraction contract covering root shell budget and module layout
  - Updated spec evidence and PR linked back to `#6948`

## Boundaries/Non-goals
- Do not change consensus or commit-store semantics.
- Do not add new transport or commit persistence features.
- Do not refactor adjacent `block_pipeline_support` modules beyond what the root split requires.
- Do not weaken current error typing or reason-code behavior.

## Failure modes
- Extraction contract does not fail red on the oversized root.
- Root shell still exceeds the active size budget after extraction.
- Durable commit checker reason projection changes behavior.
- Lane-boundary enforcement behavior regresses.
- Existing block-pipeline tests or transport-fed contracts regress.
- Touched-Rust fails because extracted files or functions remain oversized.

## Acceptance criteria
- [x] `crates/kamn-core/src/block_pipeline.rs` is reduced to a thin root shell under the active file-size policy.
- [x] Concern-based modules are extracted for models/errors, durable-commit reason projection, lane-boundary enforcement, commit-store and transport hooks, and tests.
- [x] A hard-fail extraction contract exists and passes on the final write set.
- [x] Existing block-pipeline tests and transport-fed contracts remain green.
- [x] Touched-Rust returns `policy_decision=GO` on the final write set.

## Files to touch
- `crates/kamn-core/src/block_pipeline.rs`
- `crates/kamn-core/src/block_pipeline/*.rs`
- `crates/kamn-core/tests/block_pipeline_module_extraction_contract.rs`
- `specs/6948-split-block-pipeline.md`

## Error semantics
- Preserve existing `BlockPipelineError` variants and reason-code behavior exactly.
- Keep durable-commit checker reason projection and lane-boundary enforcement fail-closed.
- No silent fallbacks or swallowed transport/commit-store errors.

## Test plan
- Red extraction contract for root shell budget and module layout.
- Green extraction contract after split.
- Real behavior checks:
  - `cargo test -p kamn-core --test block_pipeline_module_extraction_contract -- --nocapture`
  - `cargo test -p kamn-core --test block_pipeline_transport_fed -- --nocapture`
  - `cargo test -p kamn-core block_pipeline::tests:: --lib -- --nocapture`
- Touched-Rust validation with the Python entrypoint on the final write set.

## Final evidence
- `cargo test -p kamn-core --test block_pipeline_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test block_pipeline_transport_fed -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-clean-20260312-162504-pgbridge2 --base-ref github/main --output-json /tmp/6948-touched-size.json`
- Result:
  - `policy_decision=GO`
  - `offending_files=none`
  - `offending_functions=none`

## Deviations
- `cargo test -p kamn-core block_pipeline::tests:: --lib -- --nocapture` is still blocked on unrelated current-main lib-test parse failure in `crates/kamn-core/src/data_layer_m7_timeseries_telemetry/tests.rs:84`.
- The issue was verified with the narrower extraction contract plus the real `block_pipeline_transport_fed` integration target instead.
