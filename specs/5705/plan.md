# Plan: #5705 Restore R52 Green-Main Quality Gates for Review Markers and Pre-Merge Workspace Testing

## Approach
1. Capture RED baseline for marker parse failures using `release_review_activity_ratio_docs_contract`.
2. Add RED workflow contract tests that require an explicit workspace pre-merge test command in `ci-fast-gate`.
3. Fix R51 marker formatting and implement a dedicated pre-merge workspace test gate job in `.github/workflows/ci-fast-gate.yml`.
4. Re-run targeted tests for marker and workflow contracts, then portable-agent regressions (`kamn-cli`, `kamn-mcp-server`).
5. Run repository verification gates (`cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`).

## Affected Modules
- `docs/review/gaps-and-issues-r51.md`
- `.github/workflows/ci-fast-gate.yml`
- `crates/kamn-core/tests/ci_fast_gate_workspace_premerge_contract.rs` (new)
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5705/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: pre-merge workspace gate increases PR latency.
  Mitigation: keep the gate as a dedicated job with bounded retry and no unrelated deep-lane expansions.

- Risk: workflow drift introduces command variance not caught by tests.
  Mitigation: assert for the exact required workspace command markers and a stable job/step contract.

- Risk: docs-only PR cost increase from full Rust test gate.
  Mitigation: accept the tradeoff as intentional quality hardening for green-main integrity.

## Interfaces / Contracts
- CI contract: `ci-fast-gate` must include a pre-merge workspace test command:
  `cargo test --workspace --locked --all-features --no-fail-fast`.
- Release-review marker contract: R51 sections 5.3 and 5.4 must expose plain `key=value` marker lines without markdown wrappers.

## ADR
- Not required: workflow/test hardening with no dependency, protocol, or architecture change.
