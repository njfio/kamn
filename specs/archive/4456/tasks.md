# Tasks: Issue #4456

Status: Completed
Issue: #4456

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Extend `scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh` with docs drift
  fixture assertions.
- Run:
  - `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
- Expect RED before fixture/assertion updates are completed.

T2 (RED, Unit/Regression/Conformance):
- Extend `scripts/ci/test_check_workspace_license_policy.sh` with malformed/package-missing and
  missing-manifest drift assertions.
- Run:
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
- Expect fail-closed coverage for malformed/missing metadata scenarios.

T3 (RED/GREEN, Docs/Regression):
- Extend `crates/kamn-core/tests/release_gonogo_checklist_docs.rs` with dependency/license drift
  gate assertions.
- Update `docs/foundation/release-gonogo-checklist.md` with matching markers.
- Run:
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`

T4 (Verify):
- Run scoped verification:
  - `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
    - Failed with: `expected checker to fail when README dependency references drift`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
    - Failed with:
      - `assertion failed: CHECKLIST.contains("## Dependency-License Metadata/Docs Mismatch Gate (Issue #4456)")`

- GREEN command/output:
  - `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
    - Passed: `kamn-core live-https dependency posture checker tests passed.`
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
    - Passed: `workspace license policy checker tests passed.`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
    - Passed
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed
  - `cargo clippy -p kamn-node -- -D warnings`
    - Passed

- Regression summary:
  - Dependency posture tests now fail closed for additional docs-drift acceptance scenarios.
  - Workspace license policy tests now fail closed for malformed/missing metadata structures.
  - Release checklist docs contract now fails closed on dependency/license mismatch gate drift.
