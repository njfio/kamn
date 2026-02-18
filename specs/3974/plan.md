# Plan — Issue #3974

## Approach

1. Extend `check_kamn_core_missing_docs_policy.sh` to require `bootstrap`, `key_recovery`, and `kolme_runtime_commit` in graduated fixture.
2. Expand `test_check_kamn_core_missing_docs_policy.sh`:
   - add drift regressions by removing each first-batch module from graduated fixture.
   - add allowlist bypass regressions for `bootstrap` and `key_recovery` (existing `kolme_runtime_commit` path already present).
3. Update `docs/ci/strategy.md` with explicit first-batch regression contract marker entry.
4. Run targeted contract tests and CI strategy contract checks, then fast tools regression.

## Affected Paths

- `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- `scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `docs/ci/strategy.md`
- `specs/3974/spec.md`
- `specs/3974/plan.md`
- `specs/3974/tasks.md`

## Risks / Mitigations

- Risk: Contract gets too specific to current batch composition.
  Mitigation: constrain enforcement to the explicitly documented first graduation batch only.

- Risk: New regressions overlap existing generalized overlap checks.
  Mitigation: keep general overlap check and add targeted first-batch fixture presence assertions for clearer failure semantics.

## ADR

- Not required (policy contract hardening only).
