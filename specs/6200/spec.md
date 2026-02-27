# Spec: Issue 6200 - Deduplicate Kolme JSON Helper Surface

- Issue: #6200
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: backend

## Problem Statement

`kamn-kolme` carried repeated helper implementations across `api_codec.rs`,
`notification_policy.rs`, `block_scan_policy.rs`, and `flat_json_policy.rs`
for JSON token parsing and whitespace/splitting behavior. This created
maintenance divergence risk and made parser behavior drift harder to detect.

## Scope

In scope:
1. Consolidate shared scalar JSON helpers into one shared module.
2. Remove duplicate local whitespace/split helper implementations from policy modules.
3. Add/keep regression coverage proving shared helpers remain deterministic.

Out of scope:
1. Full serde migration for Kolme payload parsing.
2. Semantic parser redesign of flat JSON policy contracts.

## Acceptance Criteria

### AC-1 Shared Helper Reuse
Given policy modules that previously redefined scalar helpers,
When code is compiled,
Then those modules reuse shared helper functions rather than local copies.

### AC-2 Duplicate Helper Functions Removed
Given the Kolme source tree,
When searching for duplicate helper names (`skip_ascii_whitespace`, `split_unquoted`),
Then only shared canonical implementations remain.

### AC-3 Dedup Regression Coverage
Given shared helper behavior,
When helper tests run,
Then quoted-delimiter splitting and whitespace skipping behavior pass deterministic assertions.

## Conformance Cases

- C-01 (AC-1, Unit): `json_scalar_policy::tests::unit_json_scalar_policy_skips_ascii_whitespace_prefix`
- C-02 (AC-2, Unit): `duplicate_helper_inventory_contracts::spec_c03_kolme_json_string_helper_is_not_duplicated_across_modules`
- C-03 (AC-3, Unit): `json_scalar_policy::tests::unit_json_scalar_policy_splits_unquoted_segments_with_quoted_delimiters`

