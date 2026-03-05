# Service Secret Rotation Runbook (Issue #6381)

runbook_schema_version=kamn.ops.service-secret-rotation-runbook.v1
runbook_contract_version=v1
runbook_issue=6381

This runbook defines deterministic rotation procedures for Service API auth keys and related runtime signer secrets.
It is documentation-only and does not alter runtime cryptography behavior.

## Scope

Covered secrets/env surfaces:

- `KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX`
- `KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX`
- `KAMN_SIGNER_PRIVATE_KEY_HEX`
- `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX`
- `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY`
- `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK` (must remain unset in production)

Ownership boundaries:

- `owner_boundary.security_team=generates new key material, approves cutover window, records custody evidence`
- `owner_boundary.runtime_ops=applies env updates, executes staged rollout and verification checklist`
- `owner_boundary.incident_commander=owns rollback go/no-go decision during failed rotation`

## Key Generation

1. Generate new service-auth private key (64 hex chars) using approved custody tooling.
2. Derive the matching compressed secp256k1 public key:

```bash
SERVICE_AUTH_PRIVATE_KEY_HEX="<new_64_hex_private_key>"
python3 -c 'from cryptography.hazmat.primitives.asymmetric import ec; from cryptography.hazmat.primitives import serialization; import os; private_key = ec.derive_private_key(int(os.environ["SERVICE_AUTH_PRIVATE_KEY_HEX"], 16), ec.SECP256K1()); print(private_key.public_key().public_bytes(serialization.Encoding.X962, serialization.PublicFormat.CompressedPoint).hex())'
```

3. Store the pair in the secrets manager for the target environment with version metadata:
   - `secret_id=service_api_auth_keypair`
   - `version=<rotation_epoch>`
   - `status=staged`

## Staged Rollout

1. Canary:
   - deploy one isolated runtime slice with both:
     - `KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX=<new_private>`
     - `KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX=<new_public>`
   - keep non-canary slices on previous key material.
2. Validate canary traffic against `/healthz` and signed request flows.
3. Expand rollout by runtime slice until all service API and caller lanes run the new pair.
4. Mark previous key version `rollback_candidate` until verification is complete.

## Rollback

Rollback trigger conditions:

- authentication error spike during staged rollout.
- deterministic signer policy failures tied to rotated secret ingestion.
- failed verification checklist item.

Rollback procedure:

1. Freeze further rollout and preserve rotation evidence logs.
2. Reapply previous stable keypair version for affected slices.
3. Confirm service recovers to pre-rotation auth behavior.
4. Record rollback outcome and corrective action in incident notes.

## Verification Checklist

Run these checks before promoting rotation from `staged` to `active`:

- `cargo test -p kamn-node main_tests::runtime_tests::regression_kolme_live_signer_key_source_policy_rejects_fallback_secret_path_with_deterministic_reason_code -- --exact --nocapture`
- `bash scripts/runtime/validate_service_api_request_auth_live.sh`
- `cargo test -p kamn-core --test service_secret_rotation_runbook_docs`

Promotion decision:

- all checklist commands must pass.
- `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK` remains unset across production-targeted lanes.
- previous key version remains recoverable until post-rotation monitoring window completes.
