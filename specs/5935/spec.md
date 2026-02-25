# Spec: Issue #5935 - Task: Eliminate high-impact code duplication classes (JSON parser, helper utilities, digest/path helpers)

- Issue: #5935
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5919

## Problem Statement
Multiple duplicated helpers and parsers increase bug surface and inconsistency risk.

## Scope
In scope:
- Create shared utilities and migrate duplicate call sites (including RFC 8259-compliant JSON string parsing).

Out of scope:
- Behavioral changes outside existing helper contracts.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Duplicated parser/helper implementations are replaced with canonical shared modules.
- AC-2: JSON parser paths support escaped unicode and reject malformed payloads consistently.
- AC-3: Regression tests prevent reintroduction of duplicate helper classes.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Duplicated parser/helper implementations are replaced with canonical shared modules.
- C-02 (Functional, AC-2): Verify JSON parser paths support escaped unicode and reject malformed payloads consistently.
- C-03 (Functional, AC-3): Verify Regression tests prevent reintroduction of duplicate helper classes.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.

## Implementation Summary
- Added canonical JSON scalar and percent-encoding helper module:
  - `crates/kamn-kolme/src/json_scalar_policy.rs`
- Added canonical MCP JSON helper module:
  - `crates/kamn-mcp-server/src/json_helpers.rs`
- Removed duplicated helper definitions from:
  - `crates/kamn-kolme/src/api_codec.rs`
  - `crates/kamn-kolme/src/block_scan_policy.rs`
  - `crates/kamn-kolme/src/endpoint_policy.rs`
  - `crates/kamn-kolme/src/flat_json_policy.rs`
  - `crates/kamn-kolme/src/notification_policy.rs`
  - `crates/kamn-kolme/src/provider_response_policy.rs`
  - `crates/kamn-mcp-server/src/protocol.rs`
  - `crates/kamn-mcp-server/src/dispatch.rs`

## Verification Evidence
- RED (expected fail before implementation):
  - `cargo test -p kamn-kolme spec_c02_provider_response_fields_support_unicode_escape_sequences -- --exact`
    - failed with `invalid json value: unsupported escape sequence`
  - `cargo test -p kamn-kolme spec_c03_kolme_json_string_helper_is_not_duplicated_across_modules -- --exact`
    - failed with `api_codec must use canonical parse_json_string helper`
  - `cargo test -p kamn-mcp-server spec_c03_mcp_json_escape_helper_is_not_duplicated_across_modules -- --exact`
    - failed with `dispatch must use canonical escape_json helper`
- GREEN/Regression:
  - `cargo test -p kamn-kolme`
  - `cargo test -p kamn-mcp-server`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-kolme --tests -- -D warnings`
  - `cargo clippy -p kamn-mcp-server --tests -- -D warnings`
  - `cargo mutants --in-diff /tmp/issue5935.diff -p kamn-kolme -p kamn-mcp-server` (8/8 caught)


## Required Test Categories
- Unit: shared helper/parser test suites
- Functional: call-site parity checks
- Integration: crates consuming shared utilities remain green
- Regression: duplicate inventory checks
- Performance: parser utility overhead non-regression

## Dependencies
- #5919
