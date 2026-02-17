# Spec: #4365 Deterministic Key-Policy and Rotation-Preflight Reason Mapping Outputs

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Checker output currently exposes a raw reason list without dedicated deterministic key-policy/rotation taxonomy metadata fields required for promotion evidence contracts.

## Scope

In scope:
- Add deterministic taxonomy constants and observed-value mapping output fields.
- Print taxonomy markers in checker output for deterministic capture.
- Keep current failure semantics unchanged.

Out of scope:
- Deployment lane orchestration changes.

## Acceptance Criteria

AC-1 Output includes deterministic taxonomy version/codes markers.
AC-2 Output includes deterministic observed taxonomy value (`none|<csv>`).
AC-3 Key policy and stale rotation reasons are reflected in observed taxonomy value.

## Conformance Cases

- C-01 (AC-1): taxonomy markers always present in JSON output.
- C-02 (AC-2): GO output yields `rotation_preflight_reason_codes_value=none`.
- C-03 (AC-3): NO-GO key-source mismatch / rotation-stalled outputs include targeted reasons.

## Success Metrics

- Checker test suite remains green with deterministic output contract assertions.
