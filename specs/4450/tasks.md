# Tasks: Issue #4450

Status: Completed
Issue: #4450

## Ordered Tasks

T1 (RED, Conformance):
- Add runtime module-boundary parity docs contract tests in
  `crates/kamn-core/tests/runtime_architecture_docs.rs` for new marker set.
- Run:
  - `cargo test -p kamn-core --test runtime_architecture_docs`
- Expect RED before docs update.

T2 (RED/GREEN, Functional/Conformance):
- Add runtime extraction module-boundary parity assertions in
  `crates/kamn-node/tests/main_module_extraction_contract.rs`.
- Run:
  - `cargo test -p kamn-node --test main_module_extraction_contract`

T3 (GREEN, Docs parity):
- Update `docs/architecture/runtime.md` with runtime module-boundary parity drift
  taxonomy and deterministic guard command markers required by T1.

T4 (Verify, Regression):
- Re-run targeted tests:
  - `cargo test -p kamn-core --test runtime_architecture_docs`
  - `cargo test -p kamn-node --test main_module_extraction_contract`
- Run scoped hygiene:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `cargo test -p kamn-core --test runtime_architecture_docs`
  - Failed with:
    - `assertion failed: DOC.contains("## Runtime Module Boundary Parity Drift Cases (Issue #4450)")`
    - `assertion failed: DOC.contains("cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture")`

- GREEN command/output:
  - `cargo test -p kamn-core --test runtime_architecture_docs` -> `6 passed; 0 failed`
  - `cargo test -p kamn-node --test main_module_extraction_contract` -> `8 passed; 0 failed`
  - `cargo fmt --check` -> pass
  - `cargo clippy -p kamn-core -- -D warnings` -> pass
  - `cargo clippy -p kamn-node -- -D warnings` -> pass

- Regression summary:
  - Added fail-closed module-boundary parity contract assertions for runtime extraction source
    ownership/delegation and runtime docs marker/command drift detection.
