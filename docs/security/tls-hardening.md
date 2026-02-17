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
  - `reason_class=stable|violation`

## Deterministic Fail-Closed Reasons

Representative fail-closed reason codes include:

- `rustls_pemfile_dependency_optional_flag_mismatch`
- `webpki_roots_dependency_missing`
- `webpki_roots_feature_mapping_missing`
- `readme_adr_link_missing`
- `ci_strategy_live_https_feature_check_missing`
- `ci_strategy_no_default_features_check_missing`
- `cargo_manifest_parse_failed`

Any drift in required dependency posture or required docs markers must fail
closed with stable reason-code output.

Regression markers:

- `Regression: #4480`
- `Regression: #4481`
- `Regression: #4107`

## Release Go/No-Go TLS Evidence Convergence Markers

When TLS evidence is wired into `scripts/deploy/generate_gonogo_evidence_bundle.sh` and
validated with `scripts/deploy/check_gonogo_evidence_policy.sh`, both command outputs must expose
deterministic TLS markers:

- `tls_evidence_reason_taxonomy_version=kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1`
- `tls_evidence_reason_codes_csv=none|<csv>`
- `tls_evidence_reason_codes_value=none|<csv>`
- `tls_evidence_gate_final_decision=GO|NO-GO`

Fail-closed drills that must remain stable:

- missing report file -> `gonogo_tls_evidence_file_missing`
- stale report mtime beyond max-age window -> `gonogo_tls_evidence_freshness_window_exceeded`
- invalid report JSON payload -> `gonogo_tls_evidence_invalid_json`

Regression markers:

- `Regression: #4298`
- `Regression: #4304`
- `Regression: #4305`
