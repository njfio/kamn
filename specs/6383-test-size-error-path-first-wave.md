# Spec: Issue #6383 - Test file size and error-path policy (first wave)

## Objective

Reduce oversized test monolith risk with enforceable thresholds and land a first-wave split for a highest-size offender while adding explicit error-path assertion policy coverage for selected `Result`-returning command surfaces.

## Inputs/Outputs

- Inputs:
  - oversized test inventory (`crates/*/tests/*.rs`)
  - highest-size offender: `crates/kamn-e2e-harness/tests/command_contract.rs`
  - command-surface APIs returning `Result` in `kamn-e2e-harness`
- Outputs:
  - enforced test-file-size policy contract with deterministic thresholds/report markers
  - first-wave command contract split into smaller files with unchanged behavior
  - explicit error-path policy contract for selected `Result`-returning command surfaces

## Boundaries/Non-goals

- In scope:
  - enforce severe-size guardrails and report soft-size inventory
  - split one highest-size offender (`command_contract.rs`) into smaller integration test files
  - add explicit fail-path coverage policy for selected command parser/verify surfaces
- Out of scope:
  - blanket rewrite of all oversized test files
  - removing existing contract assertions
  - changing runtime/application behavior

## Failure modes

- FM-1: single test file grows unchecked past severe size thresholds.
- FM-2: monolith split drifts behavior by dropping existing contract assertions.
- FM-3: `Result`-returning command surfaces lose explicit error-path assertion coverage.

## Acceptance criteria (testable booleans)

- [x] AC-1: enforceable test-file-size thresholds/reporting exist and fail on severe threshold drift.
- [x] AC-2: explicit error-path assertion policy exists for selected `Result`-returning command surfaces.
- [x] AC-3: `command_contract.rs` first-wave split lands with no behavior regression.
- [x] AC-4: split evidence and coverage notes are captured in this spec.

## Files to touch

- `specs/6383-test-size-error-path-first-wave.md`
- `crates/kamn-core/tests/test_file_size_policy.rs` (new)
- `.ci/test_file_size_policy_thresholds.env` (new)
- `fixtures/ci/test_file_size_policy_baseline.env` (new)
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs` (new)
- `crates/kamn-e2e-harness/tests/command_result_error_path_policy.rs` (new)

## Error semantics

- Policy contracts fail closed when thresholds/baselines are missing or malformed.
- Error-path policy checks fail closed when required `expect_err` assertions drift.
- Split files preserve existing deterministic failure messages for command surfaces.

## Test plan

- RED:
  - add policy tests expecting severe-size enforcement and split artifacts; confirm failures on current monolith state.
  - add explicit error-path policy contract test for selected `Result` surfaces; confirm failure before split/policy fixtures.
- GREEN:
  - add thresholds/baseline fixtures and implement severe-size policy test.
  - split `command_contract.rs` by moving verify/error-heavy cases into dedicated file(s).
  - add and satisfy explicit error-path policy contract.
- REFACTOR:
  - remove duplicated helper logic across split test files where practical.
- INTEGRATION:
  - run targeted `kamn-e2e-harness` integration tests for command contracts.
  - run new policy contracts in `kamn-core` and `kamn-e2e-harness`.

## Phase 6 integration evidence

- 2026-03-05: `cargo test -p kamn-core --test test_file_size_policy` (pass)
- 2026-03-05: `cargo test -p kamn-e2e-harness --test command_result_error_path_policy` (pass)
- 2026-03-05: `cargo test -p kamn-e2e-harness --test command_contract` (pass, 90 tests)
- 2026-03-05: `cargo test -p kamn-e2e-harness --test command_contract_verify_matrix` (pass, 21 tests)

## First-wave split evidence

- Split source: `crates/kamn-e2e-harness/tests/command_contract.rs`
  - before split: 2577 LOC
  - after split: 1835 LOC
- Split target: `crates/kamn-e2e-harness/tests/command_contract_verify_matrix.rs` (858 LOC)
- Severe oversized (`>2000` LOC) test-file inventory:
  - before split: 3 files
  - after split: 2 files
- Explicit error-path policy coverage:
  - parser/surface markers retained in `command_contract.rs`
  - verify fail-path matrix retains `execute_verify_contract` `expect_err` assertions in `command_contract_verify_matrix.rs`

## Deviations

- None.
