# Spec: #4169 Quorum Marker Parity and Tamper Rejection RED Tests

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

The rotation preflight checker contract must fail closed when quorum evidence markers drift or tamper is introduced. Current coverage is broad but does not include a focused parity case for quorum approval-count drift in otherwise valid run-mode artifacts.

## Scope

In scope:
- Add focused failing-then-passing contract tests for quorum marker parity and tamper rejection.
- Validate deterministic custody continuity tamper reasons are emitted together for quorum/custody digest mismatch.
- Update runbook compatibility docs for deployment preflight quorum marker taxonomy and tamper reasons.

Out of scope:
- External approval workflow or custody-provider integration.
- Changing runtime signer quorum policy thresholds.

## Acceptance Criteria

AC-1 Quorum marker parity drift is rejected with deterministic reason markers.
AC-2 Tampered quorum/custody continuity evidence is rejected fail-closed.
AC-3 Regression tests keep rotation preflight reason taxonomy markers stable.

## Conformance Cases

- C-01 (AC-1): if `quorum_evidence_approval_count != received_approvals`, checker returns `NO-GO` and includes `quorum_evidence_approval_count_mismatch`.
- C-02 (AC-2): if `quorum_evidence_custody_sha256_match=false` in run mode, checker returns `NO-GO` and includes both `quorum_evidence_custody_sha256_mismatch` and `custody_continuity_bypass_detected`.
- C-03 (AC-3): deployment preflight docs include deterministic rotation quorum/tamper markers and validation command references.

## Success Metrics

- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh` covers C-01 and C-02.
- No existing checker contract tests regress.
- `docs/deploy/kolme_devnet_ops.md` documents the new parity/tamper contract markers.
