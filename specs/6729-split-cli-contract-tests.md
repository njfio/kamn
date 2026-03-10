# Objective

Split `crates/kamn-node/src/main_tests/cli_contract_tests.rs` into bounded sibling modules that preserve the existing CLI contract coverage while reducing the root shell under the active size policy.

# Inputs/Outputs

## Inputs
- `crates/kamn-node/src/main_tests/cli_contract_tests.rs` at 1434 LOC on current `origin/main`
- Existing `kamn-node` CLI contract coverage for required argument validation, parse failures, `kolme-live` signer/runtime enforcement, and planning/recovery/daemon regressions
- Current `kamn-node` test entrypoint wiring for `cli_contract_tests`
- Active touched-Rust size policy

## Outputs
- `crates/kamn-node/src/main_tests/cli_contract_tests.rs` reduced to a bounded root shell
- New bounded sibling modules under `crates/kamn-node/src/main_tests/cli_contract_tests/` grouped by CLI concern
- Contract coverage that fails if the CLI root shell regresses or the extracted module layout disappears
- Updated spec evidence showing the extracted CLI contract surface still runs through the real `kamn-node` test entrypoint and passes touched-Rust size checks

# Boundaries/Non-goals

- Do not change `kamn-node` CLI runtime behavior
- Do not redesign unrelated runtime or service API tests
- Do not weaken CLI contract assertions to satisfy file-size policy
- Do not add new dependencies

# Failure modes

- `cli_contract_tests.rs` remains an oversized monolith
- extracted CLI modules are arbitrary slices rather than matching the existing contract seams
- CLI parse, signer, runtime, or regression coverage is lost during extraction
- the root contract no longer enforces the CLI module layout or root shell budget
- touched-Rust size policy fails on the issue write set

# Acceptance criteria (testable booleans)

- [ ] `crates/kamn-node/src/main_tests/cli_contract_tests.rs` is reduced to a bounded root shell under the active size policy
- [ ] CLI contract coverage is split into coherent sibling modules that reflect the existing CLI concern seams
- [ ] extracted files added by this issue remain within the active touched-Rust size policy on the issue write set
- [ ] a contract fails if the CLI root shell regresses or the extracted module layout disappears
- [ ] `cargo test -p kamn-node cli_contract_tests -- --nocapture` passes after extraction
- [ ] touched-Rust size policy returns `policy_decision=GO` on the issue write set

# Files to touch

- `specs/6729-split-cli-contract-tests.md`
- `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
- new files under `crates/kamn-node/src/main_tests/cli_contract_tests/`
- `crates/kamn-node/tests/` CLI extraction contract file(s) as needed

# Error semantics

- Preserve the current hard-fail CLI assertions for required arguments, parsing failures, `kolme-live` signer/runtime validation, and daemon/planning/recovery regressions
- New extraction contracts fail with exact missing-path, missing-module-marker, or root-budget details
- No fallback path to inline CLI helper bodies or partial CLI coverage

# Test plan

1. Add a red extraction contract requiring a bounded CLI root shell and a concern-based `cli_contract_tests/` layout.
2. Extract the CLI contract tests into sibling modules grouped by argument requirements, parse validation, `kolme-live` runtime/signer coverage, and planning/recovery/daemon regressions.
3. Run `cargo test -p kamn-node cli_contract_tests -- --nocapture`.
4. Run the CLI extraction contract.
5. Run the touched-Rust size policy on the issue write set.
6. Record final evidence and any deviations in this spec before opening the PR.
