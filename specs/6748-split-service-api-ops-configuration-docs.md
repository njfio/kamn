# Objective

Split `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` into bounded concern-based sibling modules while preserving the real docs-contract test target and keeping all touched files within the active size policy.

# Inputs/Outputs

Inputs:
- Current `service_api_ops_configuration_docs.rs` test surface on `origin/main`
- Existing docs-contract helpers already used by the test target
- Active touched-Rust size policy and extraction-contract conventions

Outputs:
- Thin root `service_api_ops_configuration_docs.rs` shell with module declarations only
- Concern-based sibling modules under `crates/kamn-core/tests/service_api_ops_configuration_docs/`
- Extraction contract enforcing root shell budget and module layout
- Updated spec evidence showing the docs-contract target remained green

# Boundaries/Non-goals

- Do not change service API runtime behavior
- Do not change docs semantics except where required to support extraction
- Do not refactor unrelated kamn-core test targets
- Do not add new dependencies

# Failure Modes

- Extraction contract does not fail when moved tests remain inline in the root file
- Extracted modules exceed touched-file budget
- The real docs-contract test target no longer compiles or no longer runs moved assertions
- Shared docs helpers become duplicated instead of centralized
- Touched-Rust size policy fails on the final write set

# Acceptance Criteria

- [ ] `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` is reduced to a root shell under 200 LOC
- [ ] Extracted sibling modules exist under `crates/kamn-core/tests/service_api_ops_configuration_docs/`
- [ ] Extracted files are grouped by configuration-doc concern rather than arbitrary line count slices
- [ ] An extraction contract fails if root-shell markers or module files regress
- [ ] `cargo test -p kamn-core --test service_api_ops_configuration_docs -- --nocapture` passes on the final branch
- [ ] Touched-Rust size policy returns `policy_decision=GO` on the final write set

# Files To Touch

- `specs/6748-split-service-api-ops-configuration-docs.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs/**`
- `crates/kamn-core/tests/service_api_ops_configuration_docs_extraction_contract.rs`

# Error Semantics

- Test contracts fail loud with direct file and marker assertions
- No silent fallback for missing module files or inline-test regressions
- Existing docs-contract failures remain explicit and unchanged; this issue only reorganizes test code

# Test Plan

1. Add a red extraction contract that asserts root shell budget, required module declarations, moved-marker absence, and extracted file presence.
2. Confirm the extraction contract fails on current `main`.
3. Extract the file into concern-based sibling modules.
4. Run:
   - `cargo test -p kamn-core --test service_api_ops_configuration_docs_extraction_contract -- --nocapture`
   - `cargo test -p kamn-core --test service_api_ops_configuration_docs -- --nocapture`
   - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json <path>`
