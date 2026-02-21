# Issue #4003 Plan — Performance CI Smoke Docs Parity

## Approach
1. Extend `scripts/ci/performance_smoke_contracts.py` to accept `--strategy-doc` and validate required docs markers and remediation map coverage.
2. Add new deterministic reason codes for docs marker parity drift and docs remediation marker missing.
3. Emit docs status markers in checker stdout and report payload.
4. Update strategy docs contract block with deterministic remediation marker map.
5. Expand shell and Rust tests to enforce docs parity and remediation behavior.

## Affected Modules
- `scripts/ci/performance_smoke_contracts.py`
- `scripts/ci/test_check_performance_thresholds.sh`
- `crates/kamn-core/tests/performance_ci_smoke_governance_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: Drift between checker expected markers and docs text format.
  - Mitigation: Centralize marker strings in checker and assert exact marker presence in docs tests.
- Risk: Reason code list mismatch across checker/docs/tests.
  - Mitigation: Keep single deterministic CSV in checker and assert docs contain identical CSV.

## Interfaces / Contracts
- Checker CLI: add optional `--strategy-doc <path>` (default `docs/ci/strategy.md`).
- Checker outputs: add
  - `performance_ci_smoke_docs_status=verified|violation`
  - `performance_ci_smoke_docs_remediation_status=verified|violation`
- Reason codes add:
  - `performance_ci_smoke_docs_marker_parity_drift`
  - `performance_ci_smoke_docs_remediation_marker_missing`

## Validation Strategy
- Red: add failing shell/Rust tests for docs drift and missing remediation markers.
- Green: implement checker + docs changes until tests pass.
- Regression: run focused suite plus formatter/lint checks required by CI contract.
