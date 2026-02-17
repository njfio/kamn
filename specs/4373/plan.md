# Issue #4373 Plan

Status: Reviewed

## Approach

1. Extend RED tests in signed-to-Kolme policy and contract-lane test scripts to require:
- simulated signing rejection
- native secp256k1 marker presence
- deterministic native signer taxonomy outputs
2. Update `check_local_signed_to_kolme_demo_policy.py` to:
- enforce native signer markers from run evidence
- validate runtime signing profile markers in summary
- emit native signer reason taxonomy version/csv/value outputs
3. Update `local_signed_to_kolme_demo_contract_lane.py` summary output to include deterministic runtime signing profile evidence values used by checker contracts.
4. Update docs and doc-contract tests for release/devnet marker requirements.

## Affected Modules

- `scripts/kolme/check_local_signed_to_kolme_demo_policy.py`
- `scripts/kolme/contracts/local_signed_to_kolme_demo_contract_lane.py`
- `scripts/kolme/test_check_local_signed_to_kolme_demo_policy.sh`
- `scripts/kolme/test_run_local_signed_to_kolme_demo_contract_lane.sh`
- `docs/planning/kolme-devnet-ops.md`
- `docs/foundation/release-gonogo-checklist.md`
- `README.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: false positives in dry-run mode for signer checks.
  Mitigation: keep run-mode-only simulated/native marker checks where evidence is expected.
- Risk: drift with existing runtime integration evidence contracts.
  Mitigation: source runtime signing profile from runtime-integration summary output and keep constants aligned with existing `kolme-fork-secp256k1-v1` marker.
- Risk: docs assertions become brittle.
  Mitigation: add concise marker assertions only for deterministic taxonomy keys/values.

## Interface Contracts

- Signed-to-Kolme policy report adds:
  - `native_signer_reason_taxonomy_version`
  - `native_signer_reason_codes_csv`
  - `native_signer_reason_codes_value`
- Signed-to-Kolme summary adds deterministic runtime signing profile evidence keys used by policy validation.
