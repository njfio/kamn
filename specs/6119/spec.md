# Spec: Issue #6119 - CLI help surface for kamn-cli

- Issue: #6119
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6100

## Problem Statement
`kamn-cli --help` currently fails with `unsupported command: --help` and exit code `2`, leaving users without usage text, command listing, or argument documentation.

## Scope
In scope:
- Provide deterministic help rendering for `kamn-cli --help`, `kamn-cli -h`, and `kamn-cli help`.
- Ensure help exits successfully (`0`) and prints usage + flags + command list.
- Add tests covering help behavior and parse-error non-regression.

Out of scope:
- Full clap/argh migration.
- Command-specific detailed manpages.
- Broad CLI parser redesign beyond help handling.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `kamn-cli --help`, `kamn-cli -h`, and `kamn-cli help` exit with code `0`.
- AC-2: Help output contains usage line, supported global flags, and command listing.
- AC-3: Existing parse-error contract for empty invocation remains unchanged (exit `2`, deterministic marker).

## Conformance Cases
- C-01 (Functional, AC-1): `kamn-cli --help` exits `0`.
- C-02 (Functional, AC-1): `kamn-cli -h` exits `0`.
- C-03 (Functional, AC-1): `kamn-cli help` exits `0`.
- C-04 (Conformance, AC-2): help stdout includes `Usage:`, `--endpoint`, `--format`, and representative command names (`send-message`, `health`).
- C-05 (Regression, AC-3): empty invocation still exits `2` with `kamn-cli parse error: missing command`.

## Success Metrics / Observable Signals
- CLI help requests no longer route to parse errors.
- `cargo test -p kamn-cli` includes passing help behavior assertions.
- Existing parse error contract test remains green.
