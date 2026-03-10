# Objective

Split `crates/kamn-e2e-harness/tests/command_contract.rs` into bounded concern-based sibling modules while preserving the real `command_contract` test target and keeping all touched files within the active size policy.

# Inputs/Outputs

Inputs:
- Current `command_contract.rs` test surface on `origin/main`
- Existing harness support module at `crates/kamn-e2e-harness/tests/support/command_contract_support.rs`
- Active touched-Rust size policy and contract-test conventions

Outputs:
- Thin root `command_contract.rs` shell with module declarations only
- Concern-based sibling modules under `crates/kamn-e2e-harness/tests/command_contract/`
- Extraction contract enforcing root shell budget and module layout
- Updated spec evidence showing target coverage remained green

# Boundaries/Non-goals

- Do not change harness command behavior
- Do not change public CLI or evidence schemas
- Do not refactor unrelated test targets
- Do not introduce new production dependencies

# Failure Modes

- Extraction contract does not fail when moved tests remain inline in the root file
- Extracted modules exceed touched-file budget
- `command_contract` target no longer compiles or no longer runs moved tests
- Shared helpers duplicated across extracted files instead of centralized support
- Touched-Rust size policy fails on the final write set

# Acceptance Criteria

- [ ] `crates/kamn-e2e-harness/tests/command_contract.rs` is reduced to a root shell under 200 LOC
- [ ] Extracted sibling modules exist under `crates/kamn-e2e-harness/tests/command_contract/`
- [ ] Extracted files are grouped by command concern rather than arbitrary line count slices
- [ ] An extraction contract fails if root-shell markers or module files regress
- [ ] `cargo test -p kamn-e2e-harness command_contract -- --nocapture` passes on the final branch
- [ ] Touched-Rust size policy returns `policy_decision=GO` on the final write set

# Files To Touch

- `specs/6746-split-command-contract-tests.md`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/command_contract/**`
- `crates/kamn-e2e-harness/tests/command_contract_extraction_contract.rs`

# Error Semantics

- Test contracts fail loud with direct file and marker assertions
- No silent fallbacks for missing module files or inline-test regressions
- Existing harness command errors remain unchanged; this issue only reorganizes test code

# Test Plan

1. Add a red extraction contract that asserts:
   - root shell budget
   - required module declarations
   - absence of moved test markers in the root file
   - presence of expected extracted files
2. Confirm the extraction contract fails on current `main`
3. Extract the file into concern-based sibling modules
4. Run:
   - `cargo test -p kamn-e2e-harness --test command_contract_extraction_contract -- --nocapture`
   - `cargo test -p kamn-e2e-harness command_contract -- --nocapture`
   - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json <path>`

# Phase 6 Evidence

- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test command_contract_extraction_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-e2e-harness --test command_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/tmp python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6746 --base-ref origin/main --output-json /tmp/6746-touched-size.json`

Result:
- extraction contract: pass
- real `command_contract` target: `90 passed, 0 failed`
- touched-Rust size policy: `policy_decision=GO`

# Deviations

- Verification used `TMPDIR=/home/n/Code/kamn/tmp` and `CARGO_TARGET_DIR=/home/n/Code/kamn/target` because `/tmp` ran out of space during compile/link on the clean clone. This did not change source inputs; it only moved transient build artifacts.
