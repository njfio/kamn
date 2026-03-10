# 6735 Split runtime contract tests

## Objective
Split `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs` into bounded concern-based modules while preserving the existing daemon runtime contract coverage for structured markers, parse/env controls, shutdown behavior, completion output, and Phase 6 projection reporting.

## Inputs/Outputs
- Input: the current runtime contract test surface in `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- Output: a thin root shell plus bounded sibling modules for the runtime contract concerns, with an extraction contract that ratchets the root shell and module layout

## Boundaries/Non-goals
- Do not change daemon runtime behavior or CLI semantics
- Do not weaken or delete current runtime contract assertions
- Do not redesign the live-postgres fixture or topology helper surfaces in this issue
- Do not touch unrelated `runtime_tests/**` or service API test files unless import wiring requires it

## Failure modes
- Root runtime contract file remains oversized after extraction
- Extracted leaf files exceed the 200 LOC policy
- Structured runtime marker or shutdown assertions drift during the move
- Runtime parse/env coverage is omitted from the new module tree
- The extracted modules compile but stop running from the real `daemon_tests` entrypoint

## Acceptance criteria
- [ ] `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs` is reduced to a thin shell under the touched-file budget
- [ ] The runtime contract surface is split into bounded modules grouped by logical concern rather than arbitrary line ranges
- [ ] An extraction contract verifies the required module layout and root-shell budget
- [ ] Existing `kamn-node` daemon runtime contract coverage still runs from the real crate test entrypoint
- [ ] The touched-Rust size policy reports `GO` for the issue write set

## Files to touch
- `specs/6735-split-runtime-contract-tests.md`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests/`
- `crates/kamn-node/tests/runtime_contract_tests_extraction_contract.rs`

## Error semantics
- Extraction contracts fail loudly with explicit missing-path, missing-marker, or size-budget assertions
- Existing runtime contract assertions keep current hard-fail behavior and must not add silent fallbacks
- Any helper rewiring must preserve the current panic/assert behavior and real daemon entrypoint execution path

## Test plan
1. Add a red extraction contract that fails while the runtime contract tests remain inline and the module tree is absent
2. Extract the runtime contract clusters into bounded sibling modules and keep the root as a thin shell
3. Run `cargo test -p kamn-node --test runtime_contract_tests_extraction_contract -- --nocapture`
4. Run `cargo test -p kamn-node daemon_tests -- --nocapture`
5. Run `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /home/n/Code/kamn/tmp/6735-touched-size.json`

## Final evidence
- Root shell: `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- Shared helper surface: `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests/support.rs`
- Extracted concern modules remain under the 200 LOC file cap after the mandatory refactor pass
- Verified on issue head:
  - `cargo test -p kamn-node --test runtime_contract_tests_extraction_contract -- --nocapture`
  - `cargo test -p kamn-node daemon_tests -- --nocapture`
  - `cargo test -p kamn-node --test main_module_extraction_contract main_module_extraction_contract_daemon_tests_decomposition_shell_markers_remain_stable -- --exact --nocapture`
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /home/n/Code/kamn/tmp/6735-touched-size-final.json`
- Final touched-Rust result: `policy_decision=GO`

## Deviations
- None
