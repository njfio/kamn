# Objective

Restore `kamn-node` observability endpoint test compilation on current `origin/main` by reintroducing the explicit imports needed by the split `observability_endpoint_tests` support and leaf modules, without changing endpoint behavior or weakening assertions.

# Inputs/Outputs

## Inputs
- Current `origin/main` fails `cargo test -p kamn-node observability_endpoint -- --nocapture`
- Broken files are limited to the split `observability_endpoint_tests` surface under `crates/kamn-node/src/main_tests/observability_endpoint_tests/`
- Active touched-Rust size policy remains in effect

## Outputs
- Explicit imports restored in the broken observability endpoint support and leaf modules
- `cargo test -p kamn-node observability_endpoint -- --nocapture` passes on the issue branch
- Contract coverage that fails if the required import markers regress from the split observability test surface
- Final evidence showing the compile and touched-Rust ratchet both pass

# Boundaries/Non-goals

- Do not resume `#6723` service API split-contract extraction in this issue
- Do not change observability endpoint runtime behavior, response rendering, or test assertions
- Do not redesign the observability endpoint module layout beyond restoring the missing imports
- Do not modify unrelated `kamn-node` main test domains

# Failure modes

- `cargo test -p kamn-node observability_endpoint -- --nocapture` still fails to compile
- import fixes rely on broad wildcard re-exports instead of explicit local imports
- touched files exceed the active touched-Rust size policy
- observability endpoint assertions or coverage are weakened during the repair

# Acceptance criteria (testable booleans)

- [ ] `cargo test -p kamn-node observability_endpoint -- --nocapture` passes on the issue branch
- [ ] the fix is limited to restoring the required imports and module visibility for the split observability endpoint test support surface
- [ ] no observability endpoint test behavior or assertions are weakened
- [ ] touched files remain within the active touched-Rust size policy on the issue write set
- [ ] touched-Rust size policy returns `policy_decision=GO` on the issue write set

# Files to touch

- `specs/6724-restore-observability-endpoint-test-imports-regression.md`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests/support.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests/support/tls_support.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests/support/transport_support.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests/async_regression_contract_tests.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests/async_regression_contract_tests/negative_path_contract_tests.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests/stream_runtime_contract_tests/stream_server_contract_tests.rs`
- `crates/kamn-node/tests/` contract file(s) as needed for import regression coverage

# Error semantics

- Preserve hard-fail compile and runtime assertions already present in the observability endpoint tests
- New regression contract checks fail with exact missing-file or missing-import-marker details
- No fallback to hidden prelude imports or widened wildcard re-export chains

# Test plan

1. Add a red regression contract that requires the explicit import markers needed by the split observability endpoint support and leaf modules.
2. Re-run `cargo test -p kamn-node observability_endpoint -- --nocapture` and confirm it fails on current `origin/main`.
3. Restore the missing imports with the minimum possible source changes.
4. Re-run the observability endpoint test target.
5. Run the touched-Rust size policy on the issue write set.
6. Record final evidence and any deviations in this spec.
