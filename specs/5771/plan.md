# Plan: Issue #5771 — Enforce Pre-Merge Workspace Cargo-Test Gate

- Issue: #5771
- Spec: `specs/5771/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Strengthen `crates/kamn-core/tests/ci_fast_gate_workspace_premerge_contract.rs` with additional fail-closed assertions for:
   - bounded retry wrapper command path,
   - retry label and attempts,
   - docs parity markers in `docs/ci/strategy.md`.
2. Update `docs/ci/strategy.md` to include a dedicated pre-merge gate bullet and exact bounded-retry workspace test command.
3. Run Red→Green loop on the targeted contract test.
4. Run formatting/lint checks for touched crates.

## Affected Modules / Files
- `.github/workflows/ci-fast-gate.yml` (verification target; no semantic change expected)
- `docs/ci/strategy.md` (documentation parity markers)
- `crates/kamn-core/tests/ci_fast_gate_workspace_premerge_contract.rs` (contract assertions)
- `specs/5771/{spec.md,plan.md,tasks.md}` (lifecycle artifacts)

## Risks and Mitigations
- Risk: docs text drift or formatting changes break strict string assertions.
  - Mitigation: assert exact canonical command strings and job key marker; keep doc marker compact and stable.
- Risk: shell-surface regression due workflow edits.
  - Mitigation: avoid workflow semantic edits unless tests prove a real gap.

## Interfaces / Contracts
- CI workflow contract: `.github/workflows/ci-fast-gate.yml` must retain `workspace-premerge-gate` job with PR-only condition and bounded retry command.
- Docs contract: `docs/ci/strategy.md` must include identical command tokenization for the pre-merge gate.

## ADR
- None required (no dependency, architecture, or protocol change).
