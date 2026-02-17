# Spec: #4356 Explicit Key-Source Enforcement and Fallback-Key Rejection

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

The local KAMN live runtime integration policy must fail closed unless signer key source is explicitly declared and fallback-key paths remain unreachable. Current checks enforce several signer constraints but do not fully require explicit key-source contract markers and deterministic key-source/fallback reason outputs.

## Scope

In scope:
- `scripts/kolme/check_local_kamn_live_runtime_integration_policy.py` key-source/fallback enforcement and deterministic reason mapping outputs.
- Contract-lane and checker-facing tests for missing explicit key-source markers and fallback leakage.
- Documentation markers for key-source/fallback reason taxonomy evidence.

Out of scope:
- External KMS integration.
- Changes to signer backend Rust APIs.

## Acceptance Criteria

AC-1 Explicit key-source markers are required fail-closed.
Given a runtime integration summary for `runtime_profile=real-node`, when key-source contract markers are missing or drifted, then policy evaluation returns `NO-GO` with deterministic key-source reasons.

AC-2 Runtime command must include explicit signer key-source marker.
Given a runtime integration summary for `runtime_profile=real-node`, when `runtime_commit_command` omits the selected key-source marker, then policy evaluation returns `NO-GO` with deterministic reason code.

AC-3 Fallback-key paths are rejected fail-closed.
Given runtime integration evidence, when fallback private key command markers or fallback signer env-presence markers are present, then policy evaluation returns `NO-GO` with deterministic fallback reason codes.

AC-4 Key-source/fallback reason mapping is deterministic and machine-readable.
Given any evaluated policy report, when output JSON is emitted, then key-source/fallback taxonomy version, supported reason codes CSV, and observed reason-codes value are present and stable.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance): `runtime_signer_key_source_contract_version` missing => `runtime_signer_key_source_contract_version_missing`.
- C-02 (AC-1, Functional/Conformance): `contracts.runtime_signer_key_source_contract_version` drift => `runtime_signer_key_source_contract_version_contract_mismatch`.
- C-03 (AC-2, Functional/Conformance): real-node summary missing `KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=<value>` in `runtime_commit_command` => `runtime_commit_signer_key_source_marker_missing`.
- C-04 (AC-3, Regression): command includes `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK=` => `runtime_commit_fallback_private_key_command_marker_detected`.
- C-05 (AC-3, Regression): summary includes `runtime_signer_fallback_private_key_present=true` => `runtime_signer_fallback_private_key_present_violation`.
- C-06 (AC-4, Contract): policy output includes deterministic key-source taxonomy markers with `key_source_reason_codes_value=none` for passing evidence and CSV for violating evidence.

## Success Metrics / Observable Signals

- Existing contract lane remains green for valid evidence.
- New negative-proof tests fail before implementation and pass after implementation.
- Policy output contains deterministic key-source taxonomy fields for GO and NO-GO cases.
- Documentation includes and preserves key-source/fallback taxonomy markers.
