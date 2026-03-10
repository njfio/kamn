# Objective

Reduce `crates/kamn-node/src/main_tests/signer_tests.rs` to a thin wiring surface by extracting signer test coverage into bounded sibling modules organized by signer concern without changing signer runtime behavior, env contract semantics, or real test execution paths.

# Inputs/Outputs

## Inputs
- `crates/kamn-node/src/main_tests/signer_tests.rs` at 1857 LOC
- Existing signer runtime/profile/preflight/backend tests already wired through `kamn-node` main test surface
- Existing touched-Rust size policy and file-size budget expectations

## Outputs
- `crates/kamn-node/src/main_tests/signer_tests.rs` reduced to <= 200 LOC or a staged cap enforced by contract while acting as a module root only
- New bounded signer test modules grouped by concern, expected seams:
  - direct signer payload and profile selection tests
  - preflight and nonce resolver tests
  - managed-external backend response and reason-code tests
- Contract coverage that fails if the root file regresses above its staged cap or the extracted module layout disappears
- Updated spec evidence covering the extracted signer test surface

# Boundaries/Non-goals

- Do not change signer runtime behavior or public APIs
- Do not alter non-signer `kamn-node` test surfaces
- Do not add new dependencies
- Do not weaken or delete existing signer assertions to satisfy size policy

# Failure modes

- `signer_tests.rs` remains an oversized monolith
- extracted signer tests are sliced arbitrarily instead of by concern
- extracted modules stop participating in real `kamn-node` test wiring
- signer env/preflight/backend regression coverage is lost or renamed away without replacement
- touched-Rust size policy fails on the issue write set

# Acceptance criteria

- [ ] `crates/kamn-node/src/main_tests/signer_tests.rs` is reduced to <= 200 LOC or an explicitly staged root cap enforced by contract
- [ ] signer tests are extracted into bounded sibling modules organized by coherent signer concerns
- [ ] extracted signer files remain within the intended staged file budget
- [ ] a contract test fails if the root file regresses above its staged cap or the extracted layout disappears
- [ ] focused signer test coverage still passes through the real `kamn-node` test surface after extraction
- [ ] touched-Rust size policy passes on the issue branch

# Files to touch

- `specs/6715-split-signer-tests.md`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-node/src/main_tests/signer_tests/**`
- `crates/kamn-node/src/main_tests/*contract*` as needed for extraction enforcement

# Error semantics

- Extraction preserves the current hard-fail signer, preflight, nonce, and managed-external assertion behavior
- Contract tests fail hard with exact missing-path, file-size, and staged-root-cap details
- No fallback to inline signer test bodies or alternate test wiring layouts

# Test plan

1. Add a red contract asserting the extracted signer module layout and staged root cap.
2. Extract signer tests by concern until the contract passes.
3. Run focused `kamn-node` signer tests that cover direct signing, preflight/profile behavior, and managed-external backend response paths.
4. Run touched-Rust size policy on the issue write set.

# Outcome

- [x] `crates/kamn-node/src/main_tests/signer_tests.rs` is reduced to <= 200 LOC or an explicitly staged root cap enforced by contract
- [x] signer tests are extracted into bounded sibling modules organized by coherent signer concerns
- [x] extracted signer files remain within the intended staged file budget
- [x] a contract test fails if the root file regresses above its staged cap or the extracted layout disappears
- [x] focused signer test coverage still passes through the real `kamn-node` test surface after extraction
- [x] touched-Rust size policy passes on the issue branch

# Phase 6 Evidence

- Root/signpost files:
  - `crates/kamn-node/src/main_tests/signer_tests.rs`: `6` LOC
  - `crates/kamn-node/src/main_tests/signer_tests_split_contract.rs`: `8` LOC
  - support files remained under budget:
    - direct/profile support: `99` LOC
    - managed-external support: `181` LOC
    - preflight support: `172` LOC
- Verified on the committed refactor head:
  - `cargo test -p kamn-node signer_tests_split_contract -- --nocapture`
  - `cargo test -p kamn-node signer_tests -- --nocapture`
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6715-touched-size-final.json`
- Result:
  - signer split contract: pass
  - signer test surface: 46 passed, 0 failed
  - touched-Rust size policy: `policy_decision=GO`

# Deviations

- None.
