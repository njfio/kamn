# Plan: Issue #5935 - Task: Eliminate high-impact code duplication classes (JSON parser, helper utilities, digest/path helpers)

- Issue: #5935
- Spec: `specs/5935/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: Add/extend failing tests for conformance cases defined in specs/5935/spec.md.
2. Implement: Create shared utilities and migrate duplicate call sites (including RFC 8259-compliant JSON string parsing).
3. REGRESSION: Execute targeted + scoped suite for touched modules and close all failing deltas.
4. VERIFY: Run cargo fmt --check, strict clippy, and issue-scoped tests; collect AC evidence for PR.

## Affected Modules (Implemented)
- `crates/kamn-kolme/src/`
- `crates/kamn-mcp-server/src/protocol.rs`
- `crates/kamn-mcp-server/src/dispatch.rs`
- `crates/kamn-kolme/tests/provider_response_policy_contracts.rs`
- `crates/kamn-kolme/tests/duplicate_helper_inventory_contracts.rs`
- `crates/kamn-mcp-server/tests/duplicate_helper_inventory_contract.rs`
- `docs/architecture/helper-canonicalization.md`
- `docs/security/secure-coding.md`
- `docs/architecture/README.md`

## Delivery Notes
1. Canonicalized JSON string parsing and percent-encoding for Kolme policies through shared internal helper module with unicode escape support.
2. Canonicalized JSON escaping and nested field extraction for MCP protocol/dispatch through shared internal helper module.
3. Added source-inventory regression tests to fail closed on helper re-duplication.
4. Verified crate suites, strict clippy, formatting, and in-diff mutation testing.

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5935/spec.md`.
- Upstream issue contract: GitHub issue #5935.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.
