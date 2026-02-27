# Plan: Issue #6134

## Approach
1. Add RED unit tests in `crates/kamn-cli/src/lib.rs` for unknown-flag rejection and `--` passthrough sentinel behavior.
2. Update `parse_cli_args` token loop:
   - keep current handling for `--format` and `--endpoint`
   - add `--` sentinel to pass the remaining args through unchanged
   - return an error for any other `--*` flag
3. Run scoped fmt/clippy/tests for `kamn-cli`.

## Affected Modules
- `crates/kamn-cli/src/lib.rs`

## Risks
- Risk: rejecting unknown flags might block existing ambiguous usage.
  - Mitigation: support explicit passthrough marker `--` for values that intentionally look like flags.

## Interfaces/Contracts
- Public parser contract change: unknown `--flags` now fail fast.
