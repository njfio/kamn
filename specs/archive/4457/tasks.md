# Tasks: Issue #4457

Status: Completed
Issue: #4457

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Extend `scripts/ci/test_check_workspace_license_policy.sh` with deterministic taxonomy/class and
  CI/local boundary assertions.
- Run:
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
- Expect RED before checker implementation updates.

T2 (GREEN, Implementation):
- Implement deterministic metadata-governance taxonomy and CI/local boundary reporting in
  `scripts/ci/check_workspace_license_policy.py`.

T3 (RED/GREEN, Docs/Regression):
- Add CI strategy docs-contract assertions in `crates/kamn-core/tests/ci_strategy_docs.rs`.
- Update `docs/ci/strategy.md` with metadata-governance CI/local boundary matrix markers.
- Run:
  - `cargo test -p kamn-core --test ci_strategy_docs`

T4 (Verify):
- Run scoped verification:
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
    - Failed with: `expected deterministic reason taxonomy marker on pass output`
  - `cargo test -p kamn-core --test ci_strategy_docs`
    - Failed with:
      - `assertion failed: DOC.contains(\"metadata_governance_reason_taxonomy_version=kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1\")`

- GREEN command/output:
  - `bash scripts/ci/test_check_workspace_license_policy.sh`
    - Passed: `workspace license policy checker tests passed.`
  - `cargo test -p kamn-core --test ci_strategy_docs`
    - Passed
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed
  - `cargo clippy -p kamn-node -- -D warnings`
    - Passed

- Regression summary:
  - Workspace license checker now emits deterministic metadata-governance taxonomy/class markers.
  - CI smoke/local-heavy execution boundary markers are enforced with local-heavy opt-in fail-closed
    behavior.
  - CI strategy docs contract now fails closed on metadata-governance boundary marker drift.
