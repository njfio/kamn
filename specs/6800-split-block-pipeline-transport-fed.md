# 6800 Split block pipeline transport fed

## Objective
Split `crates/kamn-core/tests/block_pipeline_transport_fed.rs` into a thin root shell plus bounded concern-based modules while preserving the existing transport-fed block pipeline test coverage.

## Inputs/Outputs
- Input: the current monolithic `block_pipeline_transport_fed.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for transport pipeline, transport event feed, canonical replay, and restart replay coverage

## Boundaries/Non-goals
- Do not change production `kamn-core` transport-fed pipeline behavior
- Do not add new runtime features or new dependencies
- Do not weaken existing coverage or delete assertions
- Do not change public APIs

## Failure modes
- Root shell remains above the staged size budget
- Extracted files exceed the 200 LOC policy
- Shared helpers stop exposing required transport/store traits or symbols
- Restart replay coverage loses deterministic duplicate/replay assertions after extraction
- Extraction contract markers drift from the real root shell layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/block_pipeline_transport_fed.rs` is a thin shell at or below 180 LOC
- [ ] Root shell wires `transport_pipeline_contract_tests.rs`, `transport_event_feed_contract_tests.rs`, `canonical_replay_contract_tests.rs`, and `restart_replay_contract_tests.rs`
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test block_pipeline_transport_fed_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test block_pipeline_transport_fed -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6800-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6800-split-block-pipeline-transport-fed.md`
- `crates/kamn-core/tests/block_pipeline_transport_fed.rs`
- `crates/kamn-core/tests/block_pipeline_transport_fed_extraction_contract.rs`
- `crates/kamn-core/tests/block_pipeline_transport_fed/**`

## Error semantics
- Tests remain fail-closed and preserve deterministic reason-code assertions
- Shared support helpers may panic in tests with explicit messages when setup fails
- No silent fallbacks or swallowed transport/store errors

## Test plan
1. Add an extraction contract that fails while the root file is still oversized or module markers/files are absent
2. Split the monolith into bounded modules and a shared support layer
3. Run the extraction contract target
4. Run the real `block_pipeline_transport_fed` target
5. Run the touched-Rust size checker against `origin/main`

## Phase 6 evidence
- `cargo test -p kamn-core --test block_pipeline_transport_fed_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test block_pipeline_transport_fed -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6800-touched-size.json`
- Final touched-Rust result: `policy_decision=GO`

## Deviations
- The clean clone initially lacked the `specs/6800-split-block-pipeline-transport-fed.md` file and had a partially materialized split tree without commits. The branch history was repaired locally before push so the final branch preserves the required `docs -> test -> feat -> refactor -> integrate` sequence.
