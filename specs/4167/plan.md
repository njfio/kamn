# Plan: #4167 Fallback Prohibition and Signer-Material RED Coverage

## Approach

1. Extend deployment preflight policy checker tests with dedicated signer-material-missing and fallback rejection mapping assertions.
2. Keep assertions deterministic and taxonomy-driven (ordered reason-code outputs).
3. Add/update docs contracts for ops configuration fail-closed markers.

## Affected Modules

- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations

- Risk: new assertions could become brittle if they depend on full reason payloads.
  - Mitigation: assert required deterministic marker presence and ordered subset mapping outputs.
- Risk: docs drift from checker outputs.
  - Mitigation: add docs-contract tests for signer-config taxonomy/version/csv markers.
