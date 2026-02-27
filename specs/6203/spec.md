# Spec: Issue #6203 - Reclassify `*_docs.rs` Tests into Governance Lint Surface

- Status: Implemented
- Priority: P2
- Parent: #6183
- Milestone: R59 Swarm Gap Closure

## Problem Statement

R59 identified that governance/doc-contract test volume distorts behavioral test surface metrics.
`*_docs.rs`/`docs_contract` tests should be treated as governance lint surface, not behavioral
runtime test-surface in shell-vs-rust ratio reporting.

## Scope

In scope:
- Classify docs/governance Rust tests separately from behavioral Rust tests in ratio policy logic.
- Emit deterministic reporting fields for docs-test count.
- Keep non-regression gate fail-closed semantics.

Out of scope:
- Deleting existing docs-contract tests.
- Full CI lane redesign.

## Acceptance Criteria

### AC-1 Deterministic Reclassification
Given ratio policy evaluation,
When rust test files are counted,
Then docs/governance tests are excluded from behavioral rust-test count and counted separately.

### AC-2 Reporting Visibility
Given optional ratio report output,
When report is generated,
Then docs/governance rust-test count is present.

### AC-3 Gate Stability
Given ratio policy tests,
When baseline/threshold checks run,
Then policy remains fail-closed and deterministic.

## Conformance Cases

- C-01 (AC-1, Unit): classification helpers correctly detect docs/governance test files.
- C-02 (AC-2, Functional): report payload includes docs-test count marker.
- C-03 (AC-3, Regression): shell test surface ratio gate remains green with updated baseline.
