# Plan: #4356 Explicit Key-Source Enforcement and Fallback-Key Rejection

## Approach

1. Add red tests in local runtime integration contract-lane test coverage for missing explicit key-source contract markers and missing key-source command marker.
2. Implement strict checks in `check_local_kamn_live_runtime_integration_policy.py`:
   - require `runtime_signer_key_source_contract_version=v1`;
   - require `contracts.runtime_signer_key_source_contract_version=v1`;
   - require real-node runtime command to include explicit signer key-source marker.
3. Add deterministic key-source/fallback reason taxonomy output fields:
   - `key_source_reason_taxonomy_version`;
   - `key_source_reason_codes_csv`;
   - `key_source_reason_codes_value`.
4. Update key-management and release/go-no-go docs with markers used by policy/report gates.
5. Validate targeted script tests and required repo gates.

## Affected Modules

- `scripts/kolme/check_local_kamn_live_runtime_integration_policy.py`
- `scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh`
- `docs/security/key-management.md`
- `docs/foundation/release-gonogo-checklist.md`
- (if required by docs contracts) `crates/kamn-core/tests/*docs.rs`

## Risks and Mitigations

- Risk: Tightened checks break existing synthetic fixtures.
  - Mitigation: Red-first targeted fixture mutation tests + keep green-path fixture unchanged.
- Risk: New taxonomy markers drift across docs/tests.
  - Mitigation: Add exact marker assertions in script and docs contract tests.

## Interfaces / Contracts

- Policy report schema remains `kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1`.
- New required output fields:
  - `key_source_reason_taxonomy_version`
  - `key_source_reason_codes_csv`
  - `key_source_reason_codes_value`
- New deterministic reasons:
  - `runtime_signer_key_source_contract_version_missing`
  - `runtime_signer_key_source_contract_version_mismatch`
  - `runtime_signer_key_source_contract_version_contract_mismatch`
  - `runtime_commit_signer_key_source_marker_missing`
