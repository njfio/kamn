# Issue #4476 Tasks

- Issue: `#4476`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add docs parity test for TLS hardening reason-class marker and capture failing evidence.
- T2 (Green): implement `reason_class` output/report field in TLS dependency-posture checker.
- T3 (Regression): extend checker tests for pass/fail reason-class marker and report-field assertions.
- T4 (Docs): update TLS hardening policy contracts with reason-class marker.
- T5 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
  - `cargo test -p kamn-core --test tls_dependency_governance_docs security_tls_hardening_doc_tracks_reason_class_marker -- --exact`
  - `cargo mutants --in-diff`

## Completion Evidence
- TLS dependency-posture checker now emits deterministic `reason_class=stable|violation` markers in stdout and JSON report payload.
- TLS hardening docs now codify reason-class normalization marker contracts.
- RED evidence:
  - `cargo test -p kamn-core --test tls_dependency_governance_docs security_tls_hardening_doc_tracks_reason_class_marker -- --exact` failed before implementation because `docs/security/tls-hardening.md` did not include `reason_class=stable|violation`.
- GREEN/verify commands passed:
  - `cargo test -p kamn-core --test tls_dependency_governance_docs security_tls_hardening_doc_tracks_reason_class_marker -- --exact`
  - `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo mutants --in-diff` (`cargo-mutants` not installed in this environment)
