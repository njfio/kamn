# Key Management Policy

## Native secp256k1 Signing Enforcement (Issue #4373)

Production Kolme live-runtime flows must use native secp256k1 signing evidence and fail closed on simulated profiles.

- Required summary markers:
  - `runtime_signing_profile_contract_version=v1`
  - `runtime_signing_profile=kolme-fork-secp256k1-v1`
- Required command marker in run-mode evidence:
  - `KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1`
- Fail-closed reasons:
  - `runtime_commit_native_signing_profile_marker_missing`
  - `runtime_commit_simulated_signing_profile_detected`
  - `runtime_signing_profile_missing`
  - `runtime_signing_profile_mismatch`

Policy checker reference:

```bash
python3 scripts/kolme/check_local_signed_to_kolme_demo_policy.py \
  --report-file /tmp/kolme-local-signed-to-kolme-demo-summary.json \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json /tmp/kolme-local-signed-to-kolme-demo-policy.json
```

Deterministic taxonomy outputs:

- `native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1`
- `native_signer_reason_codes_csv=runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch`
- `native_signer_reason_codes_value=none|runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch`
