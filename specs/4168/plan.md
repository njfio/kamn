# Plan: #4168 Deterministic Signer Config Error Mapping

## Approach

1. Add signer-config taxonomy constants and deterministic projection helper to deployment preflight policy checker.
2. Classify run-mode signer material violations into explicit reasons:
   - `signer_secret_missing`
   - `signer_secret_invalid_hex`.
3. Keep fallback signer prohibition reasons in deterministic signer-config output mapping.
4. Update `docs/ops/configuration.md` and docs-contract tests.

## Affected Modules

- `scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py`
- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations

- Risk: additive markers may surprise downstream consumers.
  - Mitigation: keep existing rotation/custody markers unchanged; add signer-config markers as additive output.
- Risk: ambiguous reason mapping when signer material is both missing and invalid.
  - Mitigation: deterministic precedence rule in checker (`missing` takes precedence over `invalid_hex`).
