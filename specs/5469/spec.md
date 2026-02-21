# Issue #5469 Spec - R49 Gaps-and-Issues Review Artifact Publication

- Status: Implemented
- Issue: #5469
- Parent: #5449
- Milestone: R49.3 Review artifact publication and baseline refresh

## Problem Statement
After multiple R49 follow-up closure waves, there is no `docs/review/gaps-and-issues-r49.md` artifact that captures the refreshed baseline and deterministic marker state.

## Scope
In scope:
- Publish `docs/review/gaps-and-issues-r49.md` with refreshed baseline markers.
- Include deterministic evidence commands and marker values for core governance state.
- Add Rust docs-contract checks for marker presence/consistency.

Out of scope:
- Revising historical R45-R48 narrative sections.
- New production feature implementation.

## Acceptance Criteria
- AC-1: R49 review artifact exists and includes deterministic markers for branch count, milestone state, and ignored-test periodic review status.
- AC-2: Marker values reflect repository state at publication capture time.
- AC-3: Docs-contract test validates marker presence and consistency.

## Conformance Cases
- C-01 (Functional, AC-1): `docs/review/gaps-and-issues-r49.md` exists and contains required marker keys.
- C-02 (Functional, AC-2): Marker values match captured command outputs in the artifact.
- C-03 (Conformance, AC-3): `cargo test -p kamn-core --test review_r49_docs_contract -- --nocapture` passes.

## Success Metrics / Observable Signals
- R49 artifact is machine-checkable and linked to issue lifecycle.
- Missing marker keys fail docs-contract tests.
- Baseline capture commands are reproducible from doc content.
