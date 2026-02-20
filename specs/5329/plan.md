# Issue #5329 Plan

## Approach
1. Audit current ignored-test inventory from source and confirm baseline parity.
2. Add explicit `disposition` notes to every ignored-test metadata entry.
3. Keep baseline inventory aligned to source truth and preserve current ignored count where promotion is deferred.
4. Verify drift/parser contracts remain green.
5. Document justified retention with linked follow-up tracking in PR/issue evidence.

## Affected Modules
- `fixtures/ci/ignored_test_inventory_metadata.json`
- `specs/5329/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: deferring promotion leaves ignored count unchanged.
  - Mitigation: require explicit per-entry disposition + linked follow-up issue reference.
- Risk: baseline/metadata drift breaks contract lanes.
  - Mitigation: regenerate inventory and run both drift + parser contract tests before commit.

## Interfaces and Contracts
- No production API changes.
- CI ignored-test inventory schema remains unchanged.
- Metadata enhancement (`disposition`) is additive and backward-compatible.
