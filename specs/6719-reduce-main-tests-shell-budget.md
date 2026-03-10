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

# Outcome

- [x] `crates/kamn-node/src/main_tests.rs` is reduced to `<= 260` LOC while remaining a shell-only module root
- [x] extracted helper files created by this issue stay within the active size policy on the touched write set
- [x] existing `main_tests` domain modules continue to compile with the extracted helper surface
- [x] `cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture` passes on the issue branch
- [x] focused `kamn-node` test coverage using the extracted helper surface still passes through the real test wiring
- [x] touched-Rust size policy returns `policy_decision=GO` on the issue write set

# Phase 6 Evidence

- Root/signpost files:
  - `crates/kamn-node/src/main_tests.rs`: `54` LOC
  - `crates/kamn-node/src/main_tests/support.rs`: `8` LOC
  - `crates/kamn-node/src/main_tests/support/env_json.rs`: `96` LOC
  - `crates/kamn-node/src/main_tests/support/http_mock.rs`: `136` LOC
  - `crates/kamn-node/tests/main_tests_shell_budget_contract.rs`: `63` LOC
- Verified on the branch head from a clean clone at `/home/n/Code/kamn-6719-verify-1773129406`:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node --test main_tests_shell_budget_contract --manifest-path /home/n/Code/kamn-6719-verify-1773129406/crates/kamn-node/Cargo.toml -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node --test main_module_extraction_contract --manifest-path /home/n/Code/kamn-6719-verify-1773129406/crates/kamn-node/Cargo.toml -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node cli_contract_tests --manifest-path /home/n/Code/kamn-6719-verify-1773129406/crates/kamn-node/Cargo.toml -- --nocapture`
  - `python3 /home/n/Code/kamn-6719-verify-1773129406/scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6719-verify-1773129406 --base-ref 8cbed28a --output-json /home/n/Code/kamn/tmp/6719-touched-size-clean-final-direct.json`
- Result:
  - `main_tests_shell_budget_contract`: `3 passed, 0 failed`
  - `main_module_extraction_contract`: `15 passed, 0 failed`
  - `cli_contract_tests`: `53 passed, 0 failed`
  - touched-Rust size policy: `policy_decision=GO`

# Deviations

- The shell wrapper `scripts/ci/check_touched_rust_size_policy.sh` resolves `KAMN_ROOT` through the primary checkout, so for clean-clone verification the direct Python checker entrypoint was used with `--repo-root` pinned to the verify clone. The underlying policy result on the branch head was `GO`.
