# Issue #4082 Plan - CI Dry-Run Lifecycle Tamper Governance and Go/No-Go Parity

## Approach
1. Add RED tests for lifecycle CI governance checker behavior and docs parity markers:
   - new Rust contract test file for checker unit/functional/integration/regression/performance.
   - docs assertions in CI strategy and ops configuration docs tests.
2. Implement checker script `scripts/ci/check_lifecycle_ci_dry_run_governance.py`:
   - parse threshold fixture and validate required keys/types.
   - validate lifecycle artifact bundle contract markers and deterministic hashes/decision.
   - validate go/no-go dry-run report schema/markers and release-governance parity markers.
   - validate CI tools fast-mode required/forbidden entries and workflow forbidden entry.
   - validate docs marker/remediation parity in strategy + ops docs.
3. Add threshold fixture `fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env`.
4. Wire checker contract test into `scripts/ci/test_ci_tools.sh` fast/full runs.
5. Add docs sections in:
   - `docs/ci/strategy.md`
   - `docs/ops/configuration.md`
6. Run targeted verification and required gates, then prepare PR with AC/test matrix and TDD evidence.

## Affected Modules
- `scripts/ci/check_lifecycle_ci_dry_run_governance.py`
- `fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env`
- `crates/kamn-core/tests/lifecycle_ci_dry_run_governance_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `scripts/ci/test_ci_tools.sh`
- `specs/4082/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker drift between checker and docs.
  - Mitigation: checker enforces marker parity in both docs and requires remediation marker coverage.
- Risk: non-deterministic output ordering from reason codes.
  - Mitigation: normalize reason codes in threshold order and emit deterministic CSV ordering.
- Risk: CI latency growth from new checker.
  - Mitigation: bounded max-seconds threshold and explicit performance test.

## Interfaces / Contracts
- Governance report schema:
  - `kamn.ci.lifecycle-ci-dry-run-governance-report.v1`
- Governance reason taxonomy:
  - `kamn.ci.lifecycle-ci-dry-run-governance-reason-taxonomy.v1`
- Lifecycle artifact schema/taxonomy expected by checker:
  - `kamn.runtime.lifecycle-artifact-integrity-evidence.v1`
  - `kamn.runtime.lifecycle-artifact-integrity-reason-taxonomy.v1`
- Go/no-go dry-run schema/taxonomy expected by checker:
  - `kamn.runtime.go-no-go-gate-report.v1`
  - `kamn.runtime.go-no-go-gate-reason-taxonomy.v1`
