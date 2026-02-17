# Spec: #4360 Red Tests for Implicit Key-Source Acceptance and Fallback Leakage

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Current test coverage does not explicitly fail when key-source contract-version markers are omitted or when runtime command evidence omits explicit key-source marker composition for real-node runs.

## Scope

In scope:
- Add failing tests in existing runtime integration contract-lane script.
- Cover both marker omission and fallback leakage paths with deterministic reasons.

Out of scope:
- Runtime lane orchestration redesign.

## Acceptance Criteria

AC-1 Missing key-source contract-version marker produces deterministic policy failure.
AC-2 Missing runtime command key-source marker produces deterministic policy failure.
AC-3 Fallback leakage negative proof remains deterministic.

## Conformance Cases

- C-01 (AC-1): remove `runtime_signer_key_source_contract_version` from summary => failure reason includes `runtime_signer_key_source_contract_version_missing`.
- C-02 (AC-2): strip `KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=<value>` from command => failure reason includes `runtime_commit_signer_key_source_marker_missing`.
- C-03 (AC-3): inject fallback key marker in command => failure reason includes `runtime_commit_fallback_private_key_command_marker_detected`.

## Success Metrics

- New tests fail against pre-change checker behavior and pass once enforcement is implemented.
