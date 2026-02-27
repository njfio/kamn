# Spec: Issue 6213 - CLI Unknown Flags Must Fail Closed

- Issue: #6213
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: backend

## Problem Statement

`kamn-cli` currently accepts unknown `--flag` tokens as passthrough arguments.
This weakens CLI contract strictness and can hide operator mistakes.

## Scope

In scope:
1. Reject unknown `-`/`--` prefixed flags during argument parsing.
2. Preserve passthrough behavior for non-flag positional command arguments.
3. Add parser regressions for unknown-flag fail-closed behavior.

Out of scope:
1. Adding new command flags.
2. Reworking command-specific argument schemas.

## Acceptance Criteria

### AC-1 Unknown Flags Rejected
Given CLI input with unsupported `--` or `-` flag tokens,
When `parse_cli_args` parses input,
Then parsing fails with deterministic `unsupported flag: <flag>` error text.

### AC-2 Positional Passthrough Preserved
Given CLI input with non-flag positional arguments after command parsing,
When `parse_cli_args` parses input,
Then positional values remain in passthrough output unchanged.

### AC-3 Regression Coverage Added
Given parser behavior for known and unknown flags,
When unit tests run,
Then unknown-flag inputs fail closed and known behavior remains stable.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6213_cli_parser_rejects_unknown_long_flag`
- C-02 (AC-1, Unit): `tests::regression_issue_6213_cli_parser_rejects_unknown_short_flag`
- C-03 (AC-2, Unit): `tests::regression_issue_6213_cli_parser_keeps_non_flag_passthrough`

