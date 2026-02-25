# Spec: Issue #5958 - Task: complete full mutation gate for #5932 networking hardening

- Issue: #5958
- Status: Reviewed (agent-authored; multi-module P2 task flagged for human review in PR)
- Type: task
- Priority: P2
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent task: #5932

## Problem Statement
Issue #5932 requires full mutation evidence for touched networking hardening scope, but current PR state lacks complete, explicit caught/escaped mutant reporting.

## Scope
In scope:
- Run full mutation analysis for #5932 touched package scope (`kamn-core`, `kamn-node`) without shard truncation.
- Capture caught/escaped totals and escaped-mutant details.
- Post mutation evidence on PR #5957.

Out of scope:
- New runtime feature implementation beyond mutation-test hardening needed to catch escaped mutants.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: A full (non-sharded) mutation run is executed for #5932 scope (`kamn-core`, `kamn-node`).
- AC-2: Mutation results include explicit caught/missed/unviable/timeout totals.
- AC-3: Escaped mutants are either fixed with tests/code or documented with concrete justification and follow-up.
- AC-4: PR #5957 contains posted mutation evidence tied to this task.

## Conformance Cases
- C-01 (Conformance, AC-1): Run mutation command for `kamn-core` and `kamn-node` without shard flags; command exits and emits outcomes.
- C-02 (Conformance, AC-2): Extract and publish mutation totals from output artifacts/logs.
- C-03 (Regression, AC-3): For each escaped mutant, either add catching coverage or publish explicit justification.
- C-04 (Functional, AC-4): PR #5957 includes a mutation evidence comment referencing #5958.

## Success Metrics / Observable Signals
- Mutation command completes and outputs deterministic totals.
- Escaped mutants count is reduced or explicitly dispositioned.
- PR #5957 has a visible mutation evidence update comment.

## Required Test Categories
- Mutation: required for this issue scope.
- Regression: required only if escaped mutants are fixed.

## Dependencies
- #5932
- PR #5957
