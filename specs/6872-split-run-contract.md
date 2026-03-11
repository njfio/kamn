# 6872-split-run-contract

## Objective
Reduce `crates/kamn-e2e-harness/src/run_contract.rs` to a bounded root shell by extracting its run-contract execution concerns into focused sibling modules while preserving current harness behavior.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-e2e-harness/src/run_contract.rs`
  - existing `kamn-e2e-harness` run-contract tests and consumers
- Outputs:
  - a thin `run_contract.rs` coordinator/root shell
  - concern-based sibling modules for the extracted execution logic
  - regression/extraction coverage proving the refactor preserves behavior

## Boundaries/Non-goals
- Do not change run-contract semantics intentionally
- Do not redesign unrelated CLI or driver behavior
- Do not add new dependencies

## Failure modes
- extraction changes run-contract behavior or output
- helper extraction leaves functions above the 25 LOC cap
- sibling modules still exceed the 200 LOC cap
- root module retains inline monolithic logic after extraction

## Acceptance criteria
- [ ] `run_contract.rs` is reduced to a thin root shell or bounded coordinator
- [ ] extracted modules are organized by concern rather than line-count slicing
- [ ] no extracted file exceeds 200 LOC
- [ ] no newly introduced helper exceeds 25 LOC
- [ ] targeted `kamn-e2e-harness` run-contract tests pass unchanged
- [ ] an extraction contract fails if the root file regresses back into a monolith

## Files to touch
- `crates/kamn-e2e-harness/src/run_contract.rs`
- sibling modules under `crates/kamn-e2e-harness/src/run_contract/`
- extraction/regression tests under `crates/kamn-e2e-harness/tests/`

## Error semantics
- existing error behavior and propagation remain unchanged
- extraction must not introduce silent fallbacks or hidden default behavior
- helper boundaries must preserve current typed error surfaces

## Test plan
- add a red extraction contract for `run_contract.rs`
- run targeted `kamn-e2e-harness` tests covering run-contract behavior
- run touched-Rust size policy on the write set

## Final Evidence
- `cargo test -p kamn-e2e-harness --test run_contract_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-e2e-harness run_contract::tests:: -- --nocapture`
- `cargo test -p kamn-e2e-harness --test command_contract -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6872-remote-origin-clean2 --base-ref origin/main --output-json /tmp/6872-remote-origin-clean2-size.json`

## Deviations
- Final touched-Rust verification used the Python entrypoint directly against a remote-origin clean clone to avoid shell-wrapper repo-root drift and local worktree contamination.
