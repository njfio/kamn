# Spec: #4170 Rotation Evidence Bundle Checks and Deterministic Custody Reason Mapping

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Rotation governance requires deterministic reason-code mapping from custody and quorum-evidence validation outcomes. The checker currently emits fail reasons, but it does not expose an explicit custody reason taxonomy output contract in policy artifacts and runbook docs.

## Scope

In scope:
- Add deterministic custody reason taxonomy outputs to deployment preflight policy checker results.
- Ensure custody mismatch and continuity-bypass reasons are mapped in stable order.
- Update release go/no-go checklist contracts with custody reason taxonomy markers.
- Add docs contract tests for checklist coverage.

Out of scope:
- Custody platform/network integration.
- Changes to approval quorum thresholds or signer profile classes.

## Acceptance Criteria

AC-1 Checker output exposes deterministic custody reason taxonomy markers.
AC-2 Custody reason-code mapping is stable across repeated runs for identical inputs.
AC-3 Integration/docs contract coverage validates end-to-end marker presence.

## Conformance Cases

- C-01 (AC-1): output JSON/stdout includes:
  - `custody_reason_taxonomy_version`
  - `custody_reason_codes_csv`
  - `custody_reason_codes_value`
- C-02 (AC-2): a report with custody digest mismatch emits `quorum_evidence_custody_sha256_mismatch,custody_continuity_bypass_detected` in deterministic taxonomy order.
- C-03 (AC-3): `docs/foundation/release-gonogo-checklist.md` contains custody reason mapping gate markers and checker command references.

## Success Metrics

- Deployment preflight checker tests pass with new custody taxonomy assertions.
- Docs contract tests for release checklist include new custody reason markers.
- Existing rotation preflight taxonomy outputs remain unchanged for backward compatibility.
