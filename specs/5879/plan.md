# Plan: Issue #5879 - Runtime-Wide Production Panic-Path Audit + Env Fallback Remediation

- Issue: #5879
- Spec: `specs/5879/spec.md`
- Last Updated: 2026-02-24

## Approach
1. Create red evidence by running checker against runtime roots and capturing current unsafe fallback violations.
2. Expand checker default root list in `scripts/ci/check_no_production_expect.py`.
3. Refactor targeted runtime env fallback callsites to explicit match-based fallback mapping.
4. Execute checker wrapper + checker test harness + targeted crate tests.

## Affected Modules
- `scripts/ci/check_no_production_expect.py`
- `crates/kamn-cli/src/commands/mod.rs`
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-mcp-server/src/main.rs`
- `specs/5879/*`

## Risks / Mitigations
- Risk: widening scan roots could fail CI due unrelated non-runtime paths.
- Mitigation: runtime-only root list excludes e2e harness and test-surface crates.
- Risk: behavior drift in env fallback defaults.
- Mitigation: keep same fallback values and apply structural refactor only.

## Interfaces / Contracts
- Preserve checker output schema and reason taxonomy version:
  - `kamn.ci.no-production-expect-report.v1`
  - `kamn.ci.production-panic-replacement-reason-taxonomy.v1`

## ADR
No ADR required (no dependency/protocol/architecture change).
