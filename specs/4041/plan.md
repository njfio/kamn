# Issue #4041 Plan

- Issue: #4041
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Approach
1. Add `scripts/runtime/api_version_policy_live_contract.py` with:
   - `run-lane`: deterministic fixture evaluation for supported/unsupported API versions.
   - `check-policy`: fail-closed report contract validation.
   - `run-contract-lane`: lane + checker + docs parity + tamper rejection.
2. Add fixture metadata file under `fixtures/runtime/` for supported-window matrix inputs.
3. Wire `exec_dispatch` wrappers and `scripts/lib/exec_registry.json` entries:
   - `validate_api_version_policy_live.sh`
   - `check_api_version_policy_live_policy.sh`
   - `validate_api_version_policy_live_contract_lane.sh`
4. Add Rust contract tests in `crates/kamn-core/tests/api_version_policy_contract.rs` for AC/C-case coverage.
5. Update docs and docs-contract tests:
   - `docs/ci/strategy.md`
   - `docs/ops/configuration.md`
   - `crates/kamn-core/tests/ci_strategy_docs.rs`
   - `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
6. Run targeted `fmt`, `clippy`, and cargo tests.

## Affected Files
- `specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md`
- `specs/4041/spec.md`
- `specs/4041/plan.md`
- `specs/4041/tasks.md`
- `fixtures/runtime/api_version_policy_fixture_matrix.txt`
- `scripts/runtime/api_version_policy_live_contract.py`
- `scripts/runtime/validate_api_version_policy_live.sh`
- `scripts/runtime/check_api_version_policy_live_policy.sh`
- `scripts/runtime/validate_api_version_policy_live_contract_lane.sh`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/api_version_policy_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`

## Risks and Mitigations
- Risk: version-policy reason codes drift between lane, checker, and docs.
  - Mitigation: docs parity checks in `run-contract-lane` plus Rust docs-contract tests.
- Risk: shell-surface growth pressure.
  - Mitigation: wrappers remain `exec_dispatch` symlinks; behavior implemented in Python + Rust tests.
- Risk: flaky runtime budget assertions.
  - Mitigation: dry-run-only bounded performance checks with conservative threshold.

## Interface Contract
- Lane schema: `kamn.runtime.api-version-policy-report.v1`
- Policy schema: `kamn.runtime.api-version-policy-policy-report.v1`
- Contract-lane schema: `kamn.runtime.api-version-policy-contract-lane-report.v1`
- Fixture schema: `kamn.runtime.api-version-policy-fixture-matrix.v1`
- Reason taxonomy marker:
  - `api_version_policy_reason_taxonomy_version=kamn.runtime.api-version-policy-reason-taxonomy.v1`

## ADR
- Not required (no new dependencies/protocol/schema beyond deterministic policy artifact contracts).
