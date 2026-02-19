# Tasks: Issue #4455

Status: Completed
Issue: #4455

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Extend `scripts/ci/test_check_no_production_expect.sh` with failing assertions for
  taxonomy/value/class/evidence markers.
- Run:
  - `bash scripts/ci/test_check_no_production_expect.sh`
- Expect RED before checker updates.

T2 (GREEN, Implementation):
- Implement deterministic taxonomy and runtime evidence normalization in
  `scripts/ci/check_no_production_expect.py`.

T3 (RED/GREEN, Docs/Regression):
- Extend `docs/security/secure-coding.md` and `docs/foundation/release-gonogo-checklist.md`
  with panic taxonomy/evidence references.
- Extend docs contract tests and run:
  - `cargo test -p kamn-core --test secure_coding_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`

T4 (Verify):
- Run scoped verification:
  - `bash scripts/ci/test_check_no_production_expect.sh`
  - `cargo test -p kamn-core --test secure_coding_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/ci/test_check_no_production_expect.sh`
    - Failed with: `expected deterministic reason taxonomy version marker for baseline checker path`
  - `cargo test -p kamn-core --test secure_coding_docs`
    - Failed with:
      - `assertion failed: DOC.contains(\"panic_replacement_reason_taxonomy_version=kamn.ci.production-panic-replacement-reason-taxonomy.v1\")`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
    - Failed with:
      - `assertion failed: CHECKLIST.contains(\"## Panic-Replacement Reason Taxonomy and Runtime Evidence Gate (Issue #4455)\")`
- GREEN command/output:
  - `bash scripts/ci/test_check_no_production_expect.sh`
    - Passed: `production expect checker tests passed.`
  - `cargo test -p kamn-core --test secure_coding_docs`
    - Passed: `1 passed; 0 failed`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
    - Passed: `65 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed
  - `cargo clippy -p kamn-node -- -D warnings`
    - Passed
- Regression summary:
  - Panic checker now emits deterministic reason taxonomy/value/class markers for pass/fail/configuration paths.
  - Runtime evidence outputs are normalized with stable status/count/files markers for auditability.
  - Secure-coding and release go/no-go docs contracts now fail closed on panic taxonomy/evidence drift.
