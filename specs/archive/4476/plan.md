# Issue #4476 Plan

- Issue: `#4476`
- Status: `Completed`

## Approach
- Add a deterministic `reason_class` field to TLS dependency-posture checker output/report.
- Capture RED evidence by adding docs parity assertion for TLS hardening reason-class marker.
- Extend existing checker tests to assert reason-class for pass and failure drift paths.
- Update TLS hardening docs to include the reason-class normalization contract marker.

## Affected Modules
- `scripts/ci/check_kamn_core_live_https_dependency_posture.py`
- `scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
- `docs/security/tls-hardening.md`
- `crates/kamn-core/tests/tls_dependency_governance_docs.rs`

## Risks and Mitigations
- Risk: marker additions could drift from docs.
- Mitigation: add docs parity test for reason-class marker.
- Risk: output marker ordering regressions in existing tests.
- Mitigation: preserve existing markers and add deterministic reason-class assertions only.

## Interface Contract
- Additive report/output marker:
  - `reason_class=stable|violation`

## ADR
- Not required.
