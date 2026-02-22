# Spec: #5704 Align `kamn-cli` Default/Output Contract with PRD JSON Semantics

- Issue: #5704
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-cli` defaults to text output and JSON mode currently wraps command output as an escaped string blob. PRD section 4.5 defines JSON-by-default output with text as an explicit opt-in, and expects deterministic structured JSON fields for command results.

## Scope
### In Scope
- Default CLI output format changes from `text` to `json`.
- Introduce deterministic structured output rendering for all command kinds.
- Preserve explicit `--format text` behavior for human-readable key/value output.
- Update CLI parser/dispatch/command activation tests for new output contract.

### Out of Scope
- Service API route changes.
- `kamn-agent-lib` business semantics.
- New CLI subcommands.

## Acceptance Criteria
### AC-1 Default format
Given a CLI invocation without `--format`,
When args are parsed,
Then `OutputFormat::Json` is selected by default.

### AC-2 Structured JSON rendering
Given command execution success,
When output format is JSON,
Then command output is valid deterministic JSON with field-level result projections (not escaped text blobs).

### AC-3 Text compatibility
Given `--format text`,
When command execution succeeds,
Then human-readable key/value output remains available and deterministic.

### AC-4 Regression stability
Given the existing CLI command surface contracts,
When output rendering changes are integrated,
Then all `kamn-cli` tests remain green.

## Conformance Cases
- C-01 (AC-1): parser default output format test enforces `Json` default.
- C-02 (AC-2): JSON output contract test validates parseable structured JSON for representative command paths.
- C-03 (AC-3): text output contract test validates preserved key/value markers.
- C-04 (AC-4): `cargo test -p kamn-cli` passes.
- C-05 (AC-4): `cargo clippy -p kamn-cli -- -D warnings` + `cargo fmt --all --check` pass.

## Success Metrics / Observable Signals
- Running `kamn-cli <command>` without `--format` produces structured JSON.
- JSON output for successful commands is parseable by downstream tooling.
- Text output remains stable when explicitly requested.
