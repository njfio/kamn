# Plan: Issue #6121

## Approach
1. Identify the duplicated helper set across the four Kolme modules.
2. Add an internal shared helper module (`json_helpers` or similar) under `kamn-kolme/src`.
3. Replace duplicate local helper implementations with shared helper calls.
4. Preserve existing signatures/semantics where possible to avoid behavioral drift.
5. Run full crate tests for `kamn-kolme`.

## Affected Modules
- `crates/kamn-kolme/src/api_codec.rs`
- `crates/kamn-kolme/src/notification_policy.rs`
- `crates/kamn-kolme/src/block_scan_policy.rs`
- `crates/kamn-kolme/src/flat_json_policy.rs`
- `crates/kamn-kolme/src/lib.rs` (module export)
- `crates/kamn-kolme/src/<new-helper>.rs`

## Risks
- Risk: subtle parsing behavior change under edge quoting cases.
  - Mitigation: preserve original logic and lock with tests.

## Interfaces/Contracts
- Internal refactor only; no public API changes expected.
