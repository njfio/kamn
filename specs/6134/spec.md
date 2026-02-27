# Spec: Issue #6134 - CLI unknown flag rejection

Status: Accepted
Issue: #6134
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`parse_cli_args` currently forwards unknown `--flags` into positional `passthrough`, so typos like `--endpont` are silently accepted and interpreted as command arguments. This hides operator mistakes and produces misleading behavior.

## Scope
In scope:
- Reject unknown CLI flags in `parse_cli_args`.
- Preserve existing support for known global flags (`--format`, `--endpoint`).
- Add explicit passthrough escape (`--`) so positional payloads that begin with `-` remain representable.
- Add unit/conformance tests for reject and passthrough behavior.

Out of scope:
- Full CLI redesign or command-specific option parsing.
- New global options beyond this remediation.

## Acceptance Criteria
- AC-1: Unknown flags beginning with `--` return a parse error instead of entering passthrough.
- AC-2: Known global flags continue to parse exactly as before.
- AC-3: `--` sentinel passes remaining tokens through unchanged, including tokens that begin with `--`.

## Conformance Cases
- C-01 (AC-1): `kamn-cli health --endpont http://localhost:8080` fails with `unsupported flag: --endpont`.
- C-02 (AC-2): `kamn-cli health --endpoint http://localhost:8080 --format text` parses with endpoint and text format.
- C-03 (AC-3): `kamn-cli send-message -- --payload-like-flag` yields passthrough containing `--payload-like-flag`.

## Success Metrics
- `cargo test -p kamn-cli parse_cli_args`
- `cargo test -p kamn-cli`
- `cargo clippy -p kamn-cli --tests -- -D warnings`
