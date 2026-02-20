# Issue #4059 Plan

- Issue: #4059
- Milestone: specs/milestones/r27-13-authorization-tenant-isolation-and-audit-integrity-governance/index.md

## Approach
1. Reuse existing audit-integrity gate implementation in `scripts/deploy/gonogo_evidence_contract.py` rather than adding new shell scripts.
2. Add a Rust contract suite (`crates/kamn-core/tests/audit_evidence_integrity_contract.rs`) that:
   - generates a go/no-go bundle from deterministic fixtures,
   - validates policy pass path,
   - validates fail-closed tamper path,
   - asserts bounded execution time for CI-safe dry-run coverage.
3. Extend `docs/ci/strategy.md` with a dedicated audit-integrity dry-run governance section referencing existing deploy commands, marker taxonomy, and fail-closed reasons.
4. Extend `crates/kamn-core/tests/ci_strategy_docs.rs` with marker assertions for the new strategy section.
5. Run targeted fmt/clippy/test commands and record shell/rust delta markers in PR body.

## Affected Files
- `specs/4059/spec.md`
- `specs/4059/plan.md`
- `specs/4059/tasks.md`
- `crates/kamn-core/tests/audit_evidence_integrity_contract.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: brittle command-output assertions across environments.
  - Mitigation: assert deterministic marker subset and parse JSON payload contracts from generated artifact files.
- Risk: runtime budget flakes in CI.
  - Mitigation: keep commands dry-run/smoke scope and use conservative bounded threshold.
- Risk: duplicating existing audit-integrity logic in another implementation path.
  - Mitigation: test through existing deploy scripts directly; avoid adding parallel checker code.

## Interface Contract
- Audit integrity gate taxonomy marker:
  - `audit_integrity_reason_taxonomy_version=kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1`
- Deterministic reason-code taxonomy:
  - `audit_integrity_reason_codes_csv=gonogo_audit_integrity_file_missing,gonogo_audit_integrity_invalid_json,gonogo_audit_integrity_schema_mismatch,gonogo_audit_integrity_status_not_ok,gonogo_audit_integrity_final_decision_not_go,gonogo_audit_integrity_policy_status_not_verified,gonogo_audit_integrity_reason_taxonomy_version_mismatch,gonogo_audit_integrity_reason_codes_csv_mismatch,gonogo_audit_integrity_freshness_window_exceeded`
- Convergence mismatch rejection phrase:
  - `audit integrity gate convergence mismatch`

## ADR
- Not required (no new dependency, protocol, schema, or runtime architecture changes).
