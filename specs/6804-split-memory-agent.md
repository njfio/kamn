# 6804 Split memory agent tests

## Objective
Split `crates/kamn-sdk/tests/memory_agent.rs` into a thin root shell plus bounded concern-based modules while preserving the current in-memory agent SDK coverage.

## Inputs/Outputs
- Input: the current `memory_agent.rs` monolithic test target on `main`
- Output: a root shell that wires bounded sibling modules for identity/message flow, task/escrow flow, artifact lifecycle, and search/channel/DID validation

## Boundaries/Non-goals
- Do not change production `kamn-sdk` memory-agent behavior
- Do not add new dependencies or features
- Do not weaken artifact, task, escrow, or DID validation assertions
- Do not change public APIs

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared metadata/DID helpers drift from the original semantics
- Artifact lifecycle coverage loses retained/expired/tombstoned state assertions
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-sdk/tests/memory_agent.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for identity/message flow, task/escrow flow, artifact lifecycle, and query/validation coverage
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-sdk --test memory_agent_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-sdk --test memory_agent -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6804-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6804-split-memory-agent.md`
- `crates/kamn-sdk/tests/memory_agent.rs`
- `crates/kamn-sdk/tests/memory_agent_extraction_contract.rs`
- `crates/kamn-sdk/tests/memory_agent/**`

## Error semantics
- Tests remain fail-closed and preserve current `SdkError` assertions
- Shared helpers may panic in tests with explicit messages when setup fails
- No silent fallbacks or swallowed SDK errors

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the test target into bounded sibling modules and shared support
3. Run the extraction contract target
4. Run the real `memory_agent` target
5. Run the touched-Rust size checker against `origin/main`

## Phase 6 evidence
- `cargo test -p kamn-sdk --test memory_agent_extraction_contract -- --nocapture`
- `cargo test -p kamn-sdk --test memory_agent -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6804-touched-size-refactor.json`
- Final touched-Rust result: `policy_decision=GO`

## Deviations
- None.
