# Plan: Issue 6213 - CLI Unknown Flags Must Fail Closed

- Issue: #6213
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Update `parse_cli_args` token handling:
   - known flags still parsed as today
   - unknown `-`/`--` prefixed tokens return deterministic errors
   - non-flag tokens remain passthrough
2. Add unit regressions in `kamn-cli/src/lib.rs` for unknown long/short flags and positional passthrough behavior.
3. Run scoped formatting/lint/tests for `kamn-cli`.

## Affected Modules

- `crates/kamn-cli/src/lib.rs`

## Risks and Mitigations

1. Risk: command payload arguments that begin with `-` may now fail.
   - Mitigation: this is intentional fail-closed behavior; payloads should be explicit values, not unknown flags.
2. Risk: help-related flags drift.
   - Mitigation: keep existing known flag list unchanged and covered by tests.

