# Issue #4479 Plan

- Issue: `#4479`
- Status: `Completed`

## Approach
- Extend `check_anti_flake_policy.sh` with deterministic merge-gate reliability taxonomy markers and normalized reason outputs.
- Add fail-closed CI smoke/local-heavy boundary checks by validating required workflow snippets in `ci-fast-gate.yml`.
- Add regression coverage for boundary drift (fixture workflow missing boundary markers).
- Update CI strategy docs and docs parity tests for deterministic merge-gate reliability marker surface.

## Affected Modules
- `scripts/ci/check_anti_flake_policy.sh`
- `scripts/ci/test_check_anti_flake_policy.sh`
- `scripts/ci/test_anti_flake_merge_gate_policy.sh`
- `scripts/ci/test_ci_strategy_contract.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: stricter boundary checks could flag intended workflow text changes.
- Mitigation: explicit deterministic reason codes make drift auditable and deliberate updates easy.
- Risk: docs drift from marker/output surface.
- Mitigation: CI docs parity tests enforce marker presence.

## Interface Contract
- Additive checker CLI option:
  - `--fast-workflow-file`
- Additive checker/report markers:
  - `anti_flake_policy_reason_taxonomy_version`
  - `anti_flake_policy_reason_codes_csv`
  - `anti_flake_policy_reason_codes_value`
  - `anti_flake_policy_reason_class`
  - `ci_smoke_local_heavy_boundary_status`

## ADR
- Not required.
