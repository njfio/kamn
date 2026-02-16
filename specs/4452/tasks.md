# Tasks: Issue #4452

Status: Completed
Issue: #4452

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add decomposition drift assertions in
  `crates/kamn-node/tests/main_module_extraction_contract.rs`.
- Run:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
- Expect RED before docs/source updates.

T2 (RED, Conformance/Regression):
- Add docs contract test file:
  - `crates/kamn-core/tests/testing_structure_docs.rs`
- Run:
  - `cargo test -p kamn-core --test testing_structure_docs`
- Expect RED before docs file/markers are created.

T3 (GREEN, Docs/Contracts):
- Create/update `docs/testing/structure.md` with deterministic decomposition drift and
  structural budget governance markers required by T2.

T4 (Verify, Regression):
- Re-run targeted suites:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
  - `cargo test -p kamn-core --test testing_structure_docs`
- Run scoped hygiene:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
    - Failed with:
      - `main_tests.rs should carry explicit decomposition drift guard marker`
  - `cargo test -p kamn-core --test testing_structure_docs`
    - Failed with:
      - `couldn't read ... docs/testing/structure.md: No such file or directory`
- GREEN command/output:
  - `cargo test -p kamn-node --test main_module_extraction_contract`
    - Passed: `9 passed; 0 failed`
  - `cargo test -p kamn-core --test testing_structure_docs`
    - Passed: `2 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed
  - `cargo clippy -p kamn-node -- -D warnings`
    - Passed
- Regression summary:
  - Added fail-closed decomposition drift/budget contract assertions for `main_tests.rs`
    and pinned deterministic structure governance markers/commands in docs.
