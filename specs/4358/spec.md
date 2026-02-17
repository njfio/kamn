# Spec: #4358 Key-Material Handling and Rotation Preflight Checker Contracts

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

The deployment preflight policy checker enforces many key-policy and rotation fail-closed rules, but it does not expose a dedicated deterministic key-policy/rotation reason taxonomy output contract. Promotion workflows need stable machine-readable reason metadata for rotate-ready and rotate-blocked evidence.

## Scope

In scope:
- `scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py` deterministic key-policy/rotation taxonomy outputs.
- RED/green tests for key-policy violations and stale rotation preflight artifact mapping.
- Documentation updates for rotation preflight evidence markers.

Out of scope:
- Full secret-store platform integration.

## Acceptance Criteria

AC-1 Key-policy violations fail closed deterministically.
AC-2 Rotation preflight artifact drift/freshness violations map to deterministic reason outputs.
AC-3 Checker output includes deterministic key-policy/rotation taxonomy fields.
AC-4 Integration tests cover rotate-ready and rotate-blocked paths with stable reason mapping.

## Conformance Cases

- C-01 (AC-3): GO output includes
  - `rotation_preflight_reason_taxonomy_version`
  - `rotation_preflight_reason_codes_csv`
  - `rotation_preflight_reason_codes_value=none`
- C-02 (AC-1/AC-2): stale rotation rehearsal drift failure includes `signer_rotation_rehearsal_drift_detected` in taxonomy value output.
- C-03 (AC-1): production key-source mismatch failure includes `signer_key_source_production_managed_external_required` in taxonomy value output.
- C-04 (AC-4): existing rotate-ready/rotate-blocked integration fixtures remain green with deterministic taxonomy outputs.

## Success Metrics

- Existing deployment preflight policy contract suite remains green.
- New taxonomy assertions fail before implementation and pass after implementation.
- Security docs include rotation preflight evidence matrix markers.
