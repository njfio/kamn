# Plan: #4357 Multi-Signer Quorum + Signature-Decision Taxonomy

## Approach

1. Add RED assertions in `scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh` for missing signature-decision taxonomy outputs and mapped quorum drift reasons.
2. Implement signature-decision taxonomy constants and output mapping in:
   - `scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py`
3. Keep existing profile/quorum fail-closed checks intact; only add deterministic evidence projection.
4. Update `docs/ops/configuration.md` with profile/quorum signature-decision marker contract.
5. Validate with targeted scripts and required repo gates.

## Affected Modules

- `scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py`
- `scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
- `docs/ops/configuration.md`
- doc contract tests if needed for new ops-doc markers.

## Risks and Mitigations

- Risk: taxonomy list mismatch with existing reason ordering.
  - Mitigation: taxonomy order is explicit and mapping derives from deterministic ordered constants.
- Risk: negative-proof test mutations unintentionally break unrelated checks.
  - Mitigation: isolate each mutation and assert targeted reason presence.

## Interfaces / Contracts

- Policy report schema remains `kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1`.
- New output fields:
  - `signature_decision_reason_taxonomy_version`
  - `signature_decision_reason_codes_csv`
  - `signature_decision_reason_codes_value`
