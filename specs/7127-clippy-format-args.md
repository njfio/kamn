# Issue 7127: Restore Strict Clippy Format Arguments

## Objective

Restore the strict workspace Clippy gate by converting five legacy positional
format arguments in the MVP demo harness to Rust captured format arguments.

## Inputs/Outputs

- Input: the existing strings and local values passed to five `format!` calls.
- Output: byte-for-byte equivalent formatted strings and errors.
- Verification: strict Clippy, formatting, and focused MVP demo harness tests.

## Boundaries/Non-goals

- Do not change runtime behavior, public APIs, dependencies, or error semantics.
- Do not clean up unrelated lint findings.
- Do not modify shell, workflow, or template surfaces.

## Failure Modes

- A positional argument remains and strict Clippy continues to fail.
- A format string changes its rendered output.
- The focused MVP demo harness contracts regress.

## Acceptance Criteria

- [ ] All five reported `uninlined_format_args` findings are removed.
- [ ] Existing formatted output remains unchanged.
- [ ] `cargo fmt --check` passes.
- [ ] Strict workspace Clippy passes with all targets and features.
- [ ] Focused MVP demo harness tests pass.

## Files to Touch

- `crates/kamn-e2e-harness/src/mvp_demo/agent_harness.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/report_markdown.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/three_agent_claim.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract/artifact.rs`
- `specs/7127-clippy-format-args.md`

## Error Semantics

No error codes, messages, propagation, or fallback behavior may change. The
agent-harness file-read error must render the same path and underlying error.

## Test Plan

### RED

Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and
observe the five `clippy::uninlined_format_args` failures. The fifth all-targets
fixture finding becomes visible only after the four library findings are fixed.

### GREEN

Use captured arguments in only the five reported `format!` calls, then rerun
strict Clippy.

### REFACTOR

Run formatting and inspect the diff for behavior-neutral output changes only.

### INTEGRATION

Run focused `kamn-e2e-harness` MVP demo tests plus the repository `make check`
gate.
