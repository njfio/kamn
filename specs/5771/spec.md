# Spec: Issue #5771 — Enforce Pre-Merge Workspace Cargo-Test Gate

- Issue: #5771
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P1
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
R52/R53 review findings required an explicit, fail-closed pre-merge gate that runs full-workspace Rust tests on pull requests. The workflow currently has the gate, but this issue remained open without lifecycle artifacts and without explicit docs-contract assertions that prevent silent drift between workflow and CI strategy documentation.

## Scope
- Validate and codify pre-merge workspace gate invariants in workflow contract tests.
- Add CI strategy documentation markers that explicitly describe the pre-merge gate command surface.
- Add docs+workflow parity assertions so future edits fail closed.
- Deliver lifecycle artifacts (`spec.md`, `plan.md`, `tasks.md`) for issue closure.

## Out of Scope
- Changing test command semantics beyond existing policy (`cargo test --workspace --locked --all-features --no-fail-fast`).
- Altering non-PR CI workflows.
- Release pipeline changes.

## Acceptance Criteria

### AC-1: Explicit pre-merge job remains present and PR-scoped
Given `.github/workflows/ci-fast-gate.yml`,
When CI workflow contracts are validated,
Then a dedicated `workspace-premerge-gate` job must exist and run only for `pull_request` events.

### AC-2: Workspace gate command surface remains deterministic
Given `.github/workflows/ci-fast-gate.yml`,
When the pre-merge gate job executes,
Then it must invoke `scripts/ci/run_with_retry.sh` with label `workspace-premerge-tests`, max attempts `2`, and command `cargo test --workspace --locked --all-features --no-fail-fast`.

### AC-3: CI strategy docs explicitly record the pre-merge gate contract
Given `docs/ci/strategy.md`,
When docs contracts run,
Then the document must declare the `workspace-premerge-gate` job and include the exact bounded-retry workspace test command.

### AC-4: Docs/workflow drift fails closed in tests
Given workflow and docs sources,
When contract tests are executed,
Then tests must fail if any AC-1..AC-3 marker is removed or modified.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | `ci_fast_gate_workspace_premerge_contract` asserts presence of `workspace-premerge-gate:` and PR-only condition. |
| C-02 | AC-2 | Functional | `ci_fast_gate_workspace_premerge_contract` asserts bounded-retry wrapper invocation with label, attempts, and workspace test command. |
| C-03 | AC-3 | Functional | `ci_fast_gate_workspace_premerge_contract` asserts `docs/ci/strategy.md` includes job marker and exact bounded-retry command string. |
| C-04 | AC-4 | Regression | Existing + new assertions fail closed when workflow/docs markers drift. |

## Success Metrics / Observable Signals
- `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract` passes with all AC assertions.
- `cargo fmt --all --check` and targeted clippy lane pass for touched tests.
- Workspace pre-merge command and docs markers remain synchronized and grep-visible.
