# Spec: Issue 6371 - Split kamn-cli `main_contract` test surface into focused files

## Objective
Reduce test-file sprawl by splitting `crates/kamn-cli/tests/main_contract.rs` into focused contract files while preserving all assertions and behavior checks.

## Inputs/Outputs
- Inputs:
  - Existing combined test file `crates/kamn-cli/tests/main_contract.rs`.
- Outputs:
  - Focused test files (new) for extraction-contract markers.
  - Slimmed `main_contract.rs` retaining only binary/runtime behavior checks.
  - Preserved assertion coverage with equivalent markers in new files.

## Boundaries/Non-goals
- Do not change runtime behavior or CLI semantics.
- Do not remove assertions without equivalent replacement.
- Do not add dependencies.

## Failure modes
- Coverage regression if marker assertions are accidentally dropped.
- Test discovery gaps if moved tests are not in `tests/` integration targets.
- `main_contract.rs` remains oversized or still contains moved blocks.

## Acceptance criteria (testable booleans)
- [x] `main_contract.rs` keeps only its focused runtime checks and excludes moved marker blocks.
- [x] New test files own moved extraction-contract marker checks.
- [x] All moved assertions still execute and pass.
- [x] `kamn-cli` contract suites remain green.

## Files to touch
- `specs/6371-kamn-cli-main-contract-test-split.md`
- `crates/kamn-cli/tests/main_contract.rs`
- `crates/kamn-cli/tests/main_module_extraction_contract.rs` (new)
- `crates/kamn-cli/tests/main_module_surface_contract.rs` (new)

## Error semantics
- No runtime error behavior changes.
- No silent fallback behavior introduced.

## Test plan
- Red:
  - Add new integration test target assertions in `main_module_extraction_contract.rs` and remove corresponding blocks from `main_contract.rs`; run targeted tests expecting initial failure until file split wiring is complete.
- Green/Refactor/Integration:
  - `cargo test -p kamn-cli --test main_contract`
  - `cargo test -p kamn-cli --test main_module_extraction_contract`
  - `cargo test -p kamn-cli --test subcommand_surface_contract`
  - `cargo test -p kamn-cli --test command_activation_contract`

## Phase 6 integration evidence
- `cargo test -p kamn-cli --test main_contract` (pass)
- `cargo test -p kamn-cli --test main_module_extraction_contract` (pass)
- `cargo test -p kamn-cli --test main_module_surface_contract` (pass)
- `cargo test -p kamn-cli --test subcommand_surface_contract` (pass)
- `cargo test -p kamn-cli --test command_activation_contract` (pass)
- `timeout 30s cargo run -p kamn-cli -- --help` (pass)
- `timeout 30s cargo run -p kamn-cli --` (expected parse-error hard-fail: `missing command`)

## Deviations
- None.
