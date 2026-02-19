# Issue #3960 Plan

- Issue: #3960
- Status: Completed
- Spec: `specs/3960/spec.md`

## Implementation Approach
1. Add a new `kamn-node` contract test targeting signer provenance/fallback docs + config parity.
2. RED: assert required CI/docs markers and guard command that are currently absent.
3. GREEN: add deterministic marker/guard-command section in `docs/ci/strategy.md` and extend `docs/ops/configuration.md` provenance/fallback marker set.
4. Add parity logic: extract runtime signer key-source policy reason-code CSV from docs and verify required reason codes are present in source.
5. Run targeted docs-contract and parity suites.

## Affected Modules
- `crates/kamn-node/tests/signer_provenance_fallback_policy_contract.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: brittle prose assertions.
  - Mitigation: assert deterministic marker keys/commands, not descriptive prose.
- Risk: duplicate marker taxonomies diverge.
  - Mitigation: parse and compare expected CSV reason markers against source/doc anchors in one contract test.

## Contracts and Interfaces
- Marker key: `runtime_signer_key_source_policy_reason_codes_csv`.
- Required reason codes:
  - `production_signer_key_source_env_local_forbidden`
  - `fallback_signer_secret_present_violation`
- Provenance markers:
  - `managed_signer_backend_response_provenance_missing`
  - `managed_signer_backend_response_provenance_malformed`
  - `managed_signer_backend_response_provenance_mismatch`

## Verification Strategy
- RED: run new contract test before docs updates; expect marker-missing failure.
- GREEN: add docs markers/commands and rerun new test.
- REGRESSION: run existing `ci_strategy_docs` and ops-configuration docs contract test.
