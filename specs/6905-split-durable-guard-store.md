# 6905-split-durable-guard-store

## Objective
Split `crates/kamn-core/src/durable_guard_store.rs` into bounded concern-based modules while preserving durable guard bundle validation, in-memory/file/sqlite snapshot store behavior, current serialization and legacy decoding behavior, channel-policy codec helpers, and the existing real behavior coverage.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/durable_guard_store.rs` production source and its downstream consumers in core/node runtime paths
- Output: a thin root shell delegating to bounded sibling modules for durable guard bundle models/errors, store backends, serialization/legacy decoding, policy codecs, and tests
- Output: a hard-fail extraction contract enforcing the new module layout and root shell budget

## Boundaries/Non-goals
- Do not change durable guard runtime semantics or snapshot payload meaning
- Do not redesign file/sqlite storage formats beyond the seams needed for extraction
- Do not add new persistence backends, dependencies, or fallback behavior
- Do not weaken or delete existing durable-guard behavior tests to make the split pass

## Failure modes
- Extraction contract passes while `durable_guard_store.rs` remains oversized or expected modules are missing
- Public durable guard store behavior drifts and breaks downstream compile or runtime behavior silently
- Bundle validation, serialization, legacy decode, or store save/load behavior changes during extraction
- Any touched extracted file exceeds the touched-Rust size policy
- Final branch still fails touched-Rust size policy

## Acceptance criteria
- [x] `crates/kamn-core/src/durable_guard_store.rs` becomes a thin root shell under the active file-size policy
- [x] bounded sibling modules exist for bundle models/errors, store backends, serialization/legacy decoding, codec helpers, and tests
- [x] public durable guard snapshot store behavior remains unchanged for in-memory, file, and sqlite lanes
- [x] a hard-fail extraction contract exists and passes
- [x] real durable-guard behavior coverage still passes after the split
- [x] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/durable_guard_store.rs`
- `crates/kamn-core/src/durable_guard_store/`
- `crates/kamn-core/tests/durable_guard_store_module_extraction_contract.rs`
- `specs/6905-split-durable-guard-store.md`

## Error semantics
- No new fallbacks or swallowed failures in durable guard snapshot save/load, validation, or decode paths
- Existing typed errors remain fail-closed and externally observable through current return types
- Invalid payloads and schema mismatches remain deterministic and explicit

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing
- Run the extraction contract target
- Run the durable guard behavior target after the split
- Run touched-Rust size policy on the final branch

## Final evidence
- `cargo test -p kamn-core --test durable_guard_store_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test durable_guard_snapshot_store -- --nocapture`
- `cargo test -p kamn-core --test durable_guard_store_integration -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6905-touched-size-green-staged.json`
- touched-Rust final result: `policy_decision=GO`

## Deviations
- Used the Python touched-Rust entrypoint directly for final verification because it operates on the current repo root without the shell-wrapper path ambiguity seen in earlier clean-clone work.
