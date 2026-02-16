# TLS Hardening and Policy Drift Contracts

This document defines deterministic policy checks and failure taxonomy markers
for `kamn-core` live HTTPS dependency posture governance.

## Live HTTPS Dependency Posture Checker

- Command:
  - `bash scripts/ci/check_kamn_core_live_https_dependency_posture.sh --output-json /tmp/kamn-core-live-https-dependency-posture-report.json`
- Report schema:
  - `schema_version=kamn.ci.kamn-core-live-https-dependency-posture-report.v1`
- Deterministic reason taxonomy version:
  - `reason_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1`
- Normalized reason markers:
  - `reason_codes_csv=none|<csv>`
  - `reason_codes_value=none|<csv>`

## Deterministic Fail-Closed Reasons

Representative fail-closed reason codes include:

- `rustls_pemfile_dependency_optional_flag_mismatch`
- `readme_adr_link_missing`
- `ci_strategy_live_https_feature_check_missing`
- `ci_strategy_no_default_features_check_missing`
- `cargo_manifest_parse_failed`

Any drift in required dependency posture or required docs markers must fail
closed with stable reason-code output.

Regression markers:

- `Regression: #4480`
- `Regression: #4481`
