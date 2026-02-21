# Issue #5467 Spec - Completed Milestone Closure Hygiene Wave

- Status: Implemented
- Issue: #5467
- Parent: #5449
- Milestone: R49.2 Completed-milestone closure hygiene wave

## Problem Statement
Milestones `#94`, `#95`, and `#96` are completed (`open_issues=0`) but remained open, creating governance-state drift and unnecessary milestone backlog noise.

## Scope
In scope:
- Verify eligible milestones have `open_issues=0`.
- Close milestones `#94`, `#95`, and `#96`.
- Publish deterministic closure-wave evidence and docs-contract checks.

Out of scope:
- Re-opening or re-parenting historical closed issues.
- Editing delivered scope inside closed milestones.

## Acceptance Criteria
- AC-1: Milestones `#94`, `#95`, and `#96` are closed and verifiably had `open_issues=0` at closure time.
- AC-2: A planning artifact records pre/post milestone state, commands, and closed milestone set deterministically.
- AC-3: Rust docs-contract tests assert the closure markers and closed milestone IDs/count.

## Conformance Cases
- C-01 (Functional, AC-1): GitHub API milestone snapshot shows `state=closed` for `94`, `95`, `96`.
- C-02 (Functional, AC-2): Planning artifact includes pre/post evidence commands and closure marker set.
- C-03 (Conformance, AC-3): `cargo test -p kamn-core --test review_r49_completed_milestone_closure_docs_contract -- --nocapture` passes.

## Success Metrics / Observable Signals
- Open milestone inventory excludes `94`, `95`, and `96`.
- Closure-wave markers are machine-parseable.
- Missing markers fail docs-contract checks.
