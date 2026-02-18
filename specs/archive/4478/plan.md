# Issue #4478 Plan

- Issue: `#4478`
- Status: `Completed`

## Approach
- Extend anti-flake policy checker to emit deterministic reason taxonomy/version markers and normalized reason CSV/value outputs.
- Add rerun-policy contract checks against `ci-fast-gate` and `ci-deep-validate` workflows for bounded retry invariants.
- Expand anti-flake policy test coverage to include rerun-policy drift and normalized output marker assertions.
- Update CI strategy docs with anti-flake/rerun-policy deterministic markers and guard via docs test.

## Affected Modules
- `scripts/ci/check_anti_flake_policy.sh`
- `scripts/ci/test_check_anti_flake_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: stricter rerun-policy checks may fail in repos with temporary workflow divergence.
- Mitigation: deterministic fail-closed reasons make drift explicit and auditable.
- Risk: reason taxonomy updates may drift from docs.
- Mitigation: add CI strategy docs parity test markers.

## Interface Contract
- Additive checker CLI options:
  - `--fast-workflow-file`
  - `--deep-workflow-file`
- Additive output/report markers:
  - `anti_flake_policy_reason_taxonomy_version`
  - `anti_flake_policy_reason_codes_csv`
  - `anti_flake_policy_reason_codes_value`
  - `anti_flake_policy_reason_class`

## ADR
- Not required.
