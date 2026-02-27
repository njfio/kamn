# Plan: Issue 6200 - Deduplicate Kolme JSON Helper Surface

- Issue: #6200
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Extend `json_scalar_policy` with reusable helper primitives needed by multiple modules.
2. Update policy modules to import shared helpers and remove local copies.
3. Add focused unit coverage for shared split helper behavior.
4. Run scoped format/lint/tests for `kamn-kolme`.

## Affected Modules

- `crates/kamn-kolme/src/json_scalar_policy.rs`
- `crates/kamn-kolme/src/flat_json_policy.rs`
- `crates/kamn-kolme/src/provider_response_policy.rs`
- `crates/kamn-kolme/src/notification_policy.rs`
- `crates/kamn-kolme/src/block_scan_policy.rs`

## Risks and Mitigations

1. Risk: changing split semantics can break provider payload parsing.
   - Mitigation: keep algorithm identical, only centralize implementation and run existing contract tests.
2. Risk: module import drift after helper extraction.
   - Mitigation: scoped clippy + full crate tests.

