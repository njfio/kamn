# Tasks: Issue #4454

Status: Completed
Issue: #4454

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Extend `scripts/ci/test_check_no_production_expect.sh` with failing assertions for
  production `panic!`, `unreachable!`, and unsafe-fallback fixtures.
- Run:
  - `bash scripts/ci/test_check_no_production_expect.sh`
- Expect RED before checker updates.

T2 (GREEN, Implementation):
- Update `scripts/ci/check_no_production_expect.py` to detect panic-style macros and unsafe
  fallback-default patterns in production paths.

T3 (RED/GREEN, Docs/Regression):
- Add secure-coding docs contract test and required doc markers.
- Run:
  - `cargo test -p kamn-core --test secure_coding_docs`

T4 (Verify):
- Run scoped verification:
  - `bash scripts/ci/test_check_no_production_expect.sh`
  - `cargo test -p kamn-core --test secure_coding_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/ci/test_check_no_production_expect.sh`
    - Failed with: `expected checker to fail when production panic! is present`
  - `cargo test -p kamn-core --test secure_coding_docs`
    - Failed with:
      - `couldn't read ... docs/security/secure-coding.md: No such file or directory`
- GREEN command/output:
  - `bash scripts/ci/test_check_no_production_expect.sh`
    - Passed: `production expect checker tests passed.`
  - `cargo test -p kamn-core --test secure_coding_docs`
    - Passed: `1 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed
  - `cargo clippy -p kamn-node -- -D warnings`
    - Passed
- Regression summary:
  - Production panic checker now fails closed for `.expect(`, `panic!`, `unreachable!`, and unsafe env fallback defaults in pre-`#[cfg(test)]` production code.
  - Secure-coding policy doc and docs-contract tests now pin panic-path reachability and unsafe-fallback failure markers.
