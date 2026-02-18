# Issue #4476 Spec

- Title: `Task: implement TLS certificate-policy checker updates with deterministic failure taxonomy`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-41-tls-governance-completion-and-anti-flake-merge-gate-reliability-contracts/index.md`
- Parent: `#4474`

## Problem Statement
TLS dependency-posture policy checks emit deterministic reason taxonomy and normalized reason codes, but currently lack a normalized reason-class marker for stable pass/fail class auditing.

## Scope
In:
- Add deterministic `reason_class` marker/report field for TLS dependency-posture checker output.
- Extend checker tests and docs parity markers to enforce stable class output.
- Preserve existing fail-closed reason taxonomy and drift checks.

Out:
- TLS transport implementation changes.
- Certificate issuance/rotation platform changes.

## Acceptance Criteria
- AC-1: TLS policy checker emits deterministic reason-class marker (`stable|violation`) in pass/fail outputs.
- AC-2: JSON report includes deterministic reason-class field aligned with policy status.
- AC-3: Docs include reason-class marker in TLS hardening policy contracts.
- AC-4: Regression tests fail closed on reason-class drift and remain green after implementation.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh` | checker stdout includes deterministic `reason_class` marker for pass/fail cases |
| C-02 | AC-2 | Functional | `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh` | report JSON includes `reason_class` consistent with status |
| C-03 | AC-3 | Docs | `cargo test -p kamn-core --test tls_dependency_governance_docs security_tls_hardening_doc_tracks_reason_class_marker -- --exact` | TLS hardening doc includes reason-class contract marker |
| C-04 | AC-4 | Regression | `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh` | drift fixtures remain fail-closed with deterministic taxonomy output |
| C-05 | AC-4 | Integration | `cargo test -p kamn-core --test tls_dependency_governance_docs` | docs parity remains synchronized with TLS governance contracts |
| C-06 | AC-1 | Performance | `bash scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh` | checker remains bounded smoke runtime |

## Test Mapping
- `scripts/ci/check_kamn_core_live_https_dependency_posture.py`
- `scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh`
- `docs/security/tls-hardening.md`
- `crates/kamn-core/tests/tls_dependency_governance_docs.rs`

## Success Metrics
- TLS checker output includes deterministic reason class (`stable|violation`) alongside existing taxonomy/csv/value markers.
- JSON report includes `reason_class` with pass/fail-consistent values.
- Tests and docs parity prevent regression of reason-class normalization contracts.
