# 6826 Split stdio protocol contract tests

## Objective
Split `crates/kamn-mcp-server/tests/stdio_protocol_contract.rs` into a thin root shell plus bounded concern modules while preserving the existing stdio protocol contract coverage.

## Inputs/Outputs
- Input: the current monolithic `stdio_protocol_contract.rs` test target on `main`
- Output: a root shell that wires bounded sibling modules for protocol support, framed initialization/tool inventory checks, tool-dispatch success cases, and error/compatibility cases

## Boundaries/Non-goals
- Do not change production `kamn-mcp-server` stdio behavior
- Do not add new dependencies
- Do not weaken or delete current assertions
- Do not alter public APIs or runtime error semantics

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared support drifts from the current backend framing behavior
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-mcp-server/tests/stdio_protocol_contract.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for the current stdio protocol contract concerns and shared support
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-mcp-server --test stdio_protocol_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-mcp-server --test stdio_protocol_contract_extraction_contract -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/stdio-protocol-contract-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6826-split-stdio-protocol-contract.md`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract.rs`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract_extraction_contract.rs`
- `crates/kamn-mcp-server/tests/stdio_protocol_contract/**`

## Error semantics
- Tests remain fail-closed and preserve the current panic/assert behavior for protocol drift
- Shared helpers may panic with explicit messages when framed responses or JSON payloads violate the contract
- No silent fallbacks or weakened contract assertions are introduced

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the target into bounded sibling modules plus shared support
3. Run the extraction contract target
4. Run the real `stdio_protocol_contract` target
5. Run the touched-Rust size checker against `origin/main`
