# Spec: #5705 Restore R52 Green-Main Quality Gates for Review Markers and Pre-Merge Workspace Testing

- Issue: #5705
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
R52 identified two quality regressions that broke green-main guarantees:
1. `docs/review/gaps-and-issues-r51.md` wraps required marker lines in markdown backticks, so `release_review_activity_ratio_docs_contract` fails to parse required keys.
2. PR fast-gate scope selection does not guarantee a full `cargo test --workspace` run on every pull request, allowing whole-workspace regressions to reach `main`.

## Scope
### In Scope
- Remove markdown backtick wrapping from R51 marker lines in sections 5.3 and 5.4.
- Add a pre-merge CI gate in `.github/workflows/ci-fast-gate.yml` that runs `cargo test --workspace --locked --all-features --no-fail-fast` on pull requests.
- Add a deterministic contract test that fails if the workspace pre-merge gate command is removed or changed.
- Validate impacted portable-agent slices remain green (`kamn-cli`, `kamn-mcp-server`) after the quality-gate hardening.

### Out of Scope
- New command/tool features in `kamn-cli`, `kamn-mcp-server`, or `kamn-agent-lib`.
- Non-R52 milestone governance cleanup (spec-volume and branch-volume process policy).
- Deep-nightly workflow redesign beyond pre-merge gate enforcement.

## Acceptance Criteria
### AC-1 Marker formatting fix
Given `docs/review/gaps-and-issues-r51.md` marker sections 5.3 and 5.4,
When release-review marker tests parse marker lines,
Then required marker keys are discovered and both activity-ratio contract tests pass.

### AC-2 Pre-merge workspace test gate
Given any pull request execution of `ci-fast-gate`,
When the workflow runs,
Then it includes an explicit workspace-level command equivalent to:
`cargo test --workspace --locked --all-features --no-fail-fast`.

### AC-3 Workflow drift contract
Given a workflow edit that removes or mutates the workspace gate command,
When contract tests run,
Then the contract fails deterministically with a stable reason marker.

### AC-4 Regression stability
Given the R52 portable-agent deliveries,
When quality-gate hardening is integrated,
Then `kamn-cli` and `kamn-mcp-server` contract suites remain green.

## Conformance Cases
- C-01 (AC-1): `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract`.
- C-02 (AC-2, AC-3): `cargo test -p kamn-core --test ci_fast_gate_workspace_premerge_contract`.
- C-03 (AC-4): `cargo test -p kamn-cli`.
- C-04 (AC-4): `cargo test -p kamn-mcp-server`.
- C-05 (AC-1..AC-4): `cargo fmt --all --check`.
- C-06 (AC-1..AC-4): `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

## Success Metrics / Observable Signals
- R51 marker-format failures are resolved without parser exceptions.
- `ci-fast-gate` enforces a full workspace test gate before merge.
- Workflow contract tests fail closed on pre-merge gate drift.
- Portable-agent crate suites remain green post-integration.
