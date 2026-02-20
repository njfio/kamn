# Issue #5327 Plan

## Approach
1. Select 24 low-LOC kamn-core doc-contract files for consolidation.
2. Generate one harness file `docs_contract_wave3_harness.rs` with module-wrapped migrated content (preserves test names and assertions).
3. Remove migrated standalone files after content is in harness.
4. Validate include_str file-count target and run focused test/clippy suite.

## Affected Modules
- `crates/kamn-core/tests/docs_contract_wave3_harness.rs` (new)
- 24 retired files under `crates/kamn-core/tests/` (migrated into harness)
- `specs/5327/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: assertion drift during migration.
  - Mitigation: migrate by embedding exact original file bodies into per-file modules.
- Risk: harness naming collisions.
  - Mitigation: wrap each migrated file in a uniquely named Rust module.

## Interfaces and Contracts
- Assertion semantics preserved verbatim from migrated source files.
- No production code/API changes.
