# Issue #5463 Spec - Coherence Batching and Spec-Volume Guardrail Baselines

- Status: Accepted
- Issue: #5463
- Parent: #5449
- Milestone: R48.1 Spec-volume and coherence batching mitigation

## Problem Statement
R48 review leaves one open governance gap: continued growth in spec-volume ratio. Coherence hardening policy markers exist in R45, but R48 needs explicit guardrail markers and evidence hooks so future cycles can verify issue-batching and spec-volume trend posture without ad-hoc interpretation.

## Scope
In scope:
- Add deterministic spec-volume guardrail markers to R48 review docs.
- Add deterministic coherence-batching carry-forward markers to R48 review docs.
- Add docs-contract tests that fail when guardrail markers drift or disappear.

Out of scope:
- Deleting or rewriting historical spec directories.
- Refactoring closed milestone issue trees.

## Acceptance Criteria
- AC-1: R48 review doc defines deterministic spec-volume guardrail markers (schema version, baseline, target cap/status, evidence commands).
- AC-2: R48 review doc carries explicit coherence batching target markers and links them to the existing batching policy schema.
- AC-3: Rust docs-contract tests assert marker presence and numeric coherence for new guardrail markers.

## Conformance Cases
- C-01 (Functional, AC-1): `docs/review/gaps-and-issues-r48.md` contains spec-volume guardrail marker set with deterministic values.
- C-02 (Functional, AC-2): `docs/review/gaps-and-issues-r48.md` contains coherence batching carry-forward markers (`bundle_count_min/max`, issue cap, expected reduction).
- C-03 (Conformance, AC-3): `cargo test -p kamn-core --test review_r48_spec_volume_guardrail_docs_contract -- --nocapture` passes.

## Success Metrics / Observable Signals
- Guardrail markers are machine-parseable and regression-tested.
- Missing or inconsistent markers fail docs-contract tests.
- R48 review open-gap tracking is auditable via deterministic marker keys.
