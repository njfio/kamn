# Plan: Issue #5958 - Task: complete full mutation gate for #5932 networking hardening

- Issue: #5958
- Spec: `specs/5958/spec.md`
- Status: Draft
- Last Updated: 2026-02-25

## Approach
1. RED/Inventory: confirm missing full mutation evidence for #5932 in PR #5957.
2. Execute full mutation run for touched scope (`kamn-core`, `kamn-node`) without shards.
3. Parse mutation outputs into caught/missed/unviable/timeout totals.
4. If escaped mutants exist, triage and add minimum catching tests/fixes where feasible; otherwise document explicit disposition.
5. Post evidence comment on PR #5957 and update issue process log.

## Affected Modules (Initial)
- `crates/kamn-core/**` (mutation target scope)
- `crates/kamn-node/**` (mutation target scope)
- `specs/5958/spec.md`
- `specs/5958/plan.md`
- `specs/5958/tasks.md`

## Risks + Mitigations
- Risk: full mutation run can be long/unstable.
  - Mitigation: run with explicit package scoping and bounded parallel jobs.
- Risk: escaped mutants may require broader code changes.
  - Mitigation: prioritize minimal regression tests in touched networking paths; document any deferred escapes with follow-up issue.

## Interfaces / Contracts
- Primary contract source: `specs/5958/spec.md`.
- Upstream runtime scope: #5932 and PR #5957.
- Mutation evidence is published on PR #5957 as required by issue body.

## ADR Requirement
- ADR not expected (no dependency/protocol/architecture changes planned for this task).
