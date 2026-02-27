# Spec: Issue #6121 - Kolme JSON helper deduplication

Status: Reviewed
Issue: #6121
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
Kolme JSON parsing helpers are duplicated across multiple modules (`api_codec.rs`, `notification_policy.rs`, `block_scan_policy.rs`, `flat_json_policy.rs`). The duplicated logic risks divergence and inconsistent parsing behavior.

## Scope
In scope:
- Extract shared JSON helper functions into one module within `kamn-kolme`.
- Refactor duplicated call sites to use the shared helpers.
- Add/adjust tests to guard parser behavior and prevent regression.

Out of scope:
- Full parser redesign.
- Expanding parser feature support beyond existing behavior.

## Acceptance Criteria
- AC-1: Duplicated Kolme JSON helper implementations are replaced by a shared helper module.
- AC-2: Existing behavior remains unchanged for current supported inputs.
- AC-3: Regression tests validate shared helper behavior used by multiple call sites.

## Conformance Cases
- C-01 (AC-1): `skip_ascii_whitespace` and `split_unquoted` are defined in one shared location and consumed by targeted modules.
- C-02 (AC-2): Existing contract tests for affected policies continue to pass.
- C-03 (AC-3): New or updated tests verify shared helper behavior for quoted delimiters and escaped quotes.

## Success Metrics
- `cargo test -p kamn-kolme`
