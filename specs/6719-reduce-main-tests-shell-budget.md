# Objective

Reduce `crates/kamn-node/src/main_tests.rs` below the enforced shell budget by extracting shared helper/runtime-fixture code into bounded sibling modules while preserving the existing `main_tests::*` wiring and all current test behavior.

# Inputs/Outputs

## Inputs
- `crates/kamn-node/src/main_tests.rs` at 261 LOC on `origin/main`
- `crates/kamn-node/tests/main_module_extraction_contract.rs` enforcing `main_tests.rs <= 260` and shell-only structure
- Existing `kamn-node` main test modules that import shared helpers from the `main_tests` root

## Outputs
- `crates/kamn-node/src/main_tests.rs` reduced to `<= 260` LOC and acting as a shell/root only
- New bounded shared helper module(s) under `crates/kamn-node/src/main_tests/` carrying the extracted env/mock HTTP support code
- Existing domain test modules continuing to compile and run through the same real `main_tests` wiring
- Contract/test evidence showing the full `main_module_extraction_contract` target passes on the issue branch

# Boundaries/Non-goals

- Do not change signer, service API, daemon, CLI, or observability runtime behavior
- Do not add new dependencies
- Do not weaken the `main_module_extraction_contract` budget or assertions
- Do not redesign individual domain test files beyond the imports needed to consume extracted helpers

# Failure modes

- `crates/kamn-node/src/main_tests.rs` remains above the `<= 260` shell cap
- helper extraction breaks existing imports from `main_tests::*`
- inline `#[test]` bodies or new implementation drift reappear in `main_tests.rs`
- touched-Rust size policy fails on the issue write set
- the full `main_module_extraction_contract` target still fails on the issue branch

# Acceptance criteria

- [ ] `crates/kamn-node/src/main_tests.rs` is reduced to `<= 260` LOC while remaining a shell-only module root
- [ ] extracted helper files created by this issue stay within the active size policy on the touched write set
- [ ] existing `main_tests` domain modules continue to compile with the extracted helper surface
- [ ] `cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture` passes on the issue branch
- [ ] focused `kamn-node` test coverage using the extracted helper surface still passes through the real test wiring
- [ ] `bash scripts/ci/check_touched_rust_size_policy.sh --output-json <tmpfile>` returns `policy_decision=GO`

# Files to touch

- `specs/6719-reduce-main-tests-shell-budget.md`
- `crates/kamn-node/src/main_tests.rs`
- new bounded helper module file(s) under `crates/kamn-node/src/main_tests/`
- existing `crates/kamn-node/src/main_tests/*.rs` files only as needed to update imports after extraction

# Error semantics

- Preserve current hard-fail helper behavior, including panic/assert semantics used by tests
- Do not introduce silent fallback or alternate helper implementations
- Any extracted helpers must preserve the current `main_tests` import surface or fail loudly at compile time

# Test plan

1. Add a red contract/assertion derived from `main_module_extraction_contract` expectations if needed to make the shell-budget regression explicit for this issue.
2. Extract shared helper code from `main_tests.rs` into bounded sibling support module(s).
3. Run focused `kamn-node` tests that exercise the extracted helper surface.
4. Run the full `main_module_extraction_contract` target.
5. Run touched-Rust size policy on the issue write set.
