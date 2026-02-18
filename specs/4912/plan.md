# Plan — Issue #4912

## Approach

1. Add failing tests/contracts for shell LOC hard ceiling and archive-pointer policy.
2. Implement hard ceiling checker and wire it into CI tools and fast-gate shell-surface policy chain.
3. Archive completed milestone specs into `specs/archive/` and replace active paths with pointer stubs.
4. Delete a bounded set of superseded legacy wrappers now covered by shared runner contracts.

## Affected Modules

- `scripts/ci/`
- `.github/workflows/ci-fast-gate.yml`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-17-shell-loc-reduction-plan.md`
- `specs/archive/`
- `specs/4*/`

## Risks / Mitigations

- Risk: deleting wrappers breaks selectors/docs/tests.
  Mitigation: use explicit contract tests and migrate all callsites to shared runner entrypoints first.
- Risk: archive move breaks spec path assumptions.
  Mitigation: keep deterministic pointer files at `specs/<id>/` and add policy tests.
- Risk: ceiling check false positives from symlink duplication.
  Mitigation: compute from git-tracked non-symlink `.sh` files only.

## Interfaces / Contracts

- New CI checker output contract:
  - `reason_taxonomy_version=kamn.ci.shell-loc-hard-ceiling-reason-taxonomy.v1`
  - deterministic `reason_codes` ordering
- Archive pointer format contract at `specs/<id>/ARCHIVED.md`.

## ADR

- Not required (policy/mechanics update; no dependency or protocol change).

