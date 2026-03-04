# Spec: Issue #6323 - extract main.rs report output IO helpers

## Objective

Reduce `crates/kamn-node/src/main.rs` surface by extracting report-output and stdout/stderr
writer helpers into a dedicated module, keeping runtime behavior unchanged.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-node/src/main.rs` helper functions:
    - `emit_bootstrap_report_output`
    - `write_stdout_line`
    - `write_stderr_line`
    - `write_line_to_stream`
  - extraction contract lane:
    - `crates/kamn-node/tests/main_module_extraction_contract.rs`
- Outputs:
  - new module file owning report output/stream writing logic.
  - `main.rs` delegates output operations through the new module.
  - extraction contract lane asserts removed inline helpers.

## Boundaries/Non-goals

- In scope:
  - output-helper extraction only.
  - contract tests for absence of inline output helpers in `main.rs`.
- Out of scope:
  - report data model restructuring.
  - runtime mode orchestration refactors.
  - endpoint module extraction changes.

## Failure modes

- FM-1: output helper functions remain inline in `main.rs` after extraction.
- FM-2: error output path (`main` failure branch) no longer writes deterministic stderr line.
- FM-3: bootstrap report output path changes rendered payload or stream semantics.

## Acceptance criteria (testable booleans)

- AC-1: `main.rs` no longer defines `fn emit_bootstrap_report_output(`.
- AC-2: `main.rs` no longer defines `fn write_stdout_line(` or `fn write_stderr_line(`.
- AC-3: `main.rs` no longer defines `fn write_line_to_stream(`.
- AC-4: `main.rs` declares and uses the extracted output helper module.
- AC-5: `cargo test -p kamn-node main_module_extraction_contract` passes.

## Files to touch

- `specs/6323-main-output-io-extraction.md`
- `crates/kamn-node/src/main.rs`
- `crates/kamn-node/src/output_io.rs` (new)
- `crates/kamn-node/tests/main_module_extraction_contract.rs`

## Error semantics

- Preserve current `ConfigError::RuntimeDaemonLifecycle` mapping for stream write/flush failures.
- Preserve single-line newline-terminated output behavior for stdout and stderr helper paths.

## Test plan

- RED:
  - extend extraction contract lane to assert output helpers are absent from `main.rs` and
    new module declaration exists.
  - run `cargo test -p kamn-node main_module_extraction_contract` and confirm failure.
- GREEN:
  - add `output_io.rs`, move helper implementations, wire imports/calls in `main.rs`.
- REFACTOR:
  - keep module API minimal and self-documenting.
- INTEGRATION:
  - run:
    - `cargo test -p kamn-node main_module_extraction_contract`
    - `cargo test -p kamn-node cli_tests -- --nocapture`

## Phase 6 integration evidence

- `cargo test -p kamn-node main_module_extraction_contract`:
  - pass (`12 passed, 0 failed`)
- `cargo test -p kamn-node cli_tests -- --nocapture`:
  - pass (`7 passed, 0 failed`)

## Deviations

- None.
