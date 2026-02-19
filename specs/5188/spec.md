# Issue #5188 Spec

- Title: Task: implement kamn-core public API surface report and growth ratchet
- Status: Reviewed
- Priority: P2
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Problem Statement
`kamn-core` public API breadth is large and currently lacks a deterministic, in-repo ratchet that reports total/per-module surface drift and fails closed when growth exceeds approved limits.

## Scope
In:
- Add a deterministic `kamn-core` public API surface report schema produced by Rust test logic.
- Add policy enforcement with warn/fail thresholds and fail-closed unchecked-growth behavior.
- Add baseline + threshold fixtures and documentation for baseline refresh and waiver workflow.

Out:
- Public API reduction/refactor itself (this issue establishes measurement and gating only).
- Shell-script based policy runners or new workflow wrappers.

## Acceptance Criteria
- AC-1: A deterministic public API surface report is produced with schema version, total count, per-module counts, and deltas vs baseline.
- AC-2: Warn/fail ratchet policy is enforced by tests with fail-closed behavior when growth exceeds fail thresholds without a valid waiver.
- AC-3: Documentation defines baseline refresh workflow and waiver process, including mitigation issue linkage.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | `kamn-core/src/lib.rs` + module source tree + baseline fixture | Deterministic report fields render with stable schema markers and sorted module ordering |
| C-02 | AC-1 | Functional | Optional output path env var for report emission | Report artifact is written deterministically and contains required markers |
| C-03 | AC-2 | Regression | Current API counts compared to baseline + thresholds (+ optional waiver fixture) | Policy classification resolves to `within`/`warn`/`fail`, and `fail` panics when unwaived |
| C-04 | AC-3 | Functional | CI strategy docs + contract doc tests | Docs include command, baseline refresh, and waiver requirements with tracked mitigation issue marker |

## Test Mapping
- C-01/C-02 -> `crates/kamn-core/tests/public_api_surface_policy.rs`
- C-03 -> `crates/kamn-core/tests/public_api_surface_policy.rs`
- C-04 -> `crates/kamn-core/tests/ci_strategy_docs.rs`

## Success Metrics
- `cargo test -p kamn-core --test public_api_surface_policy` passes and enforces ratchet policy.
- `cargo test -p kamn-core --test ci_strategy_docs` validates documentation markers for the new gate.
- Implementation adds no new shell scripts and keeps shell LOC delta at `0`.
