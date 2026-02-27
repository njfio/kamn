# Plan: Issue #6119

## Approach
1. Add RED integration tests in `crates/kamn-cli/tests/main_contract.rs` for `--help`, `-h`, and `help` success behavior and required output markers.
2. Introduce a deterministic help text renderer in `kamn-cli` library.
3. Add a lightweight `is_help_request` classifier and invoke it in `src/main.rs` before parser dispatch.
4. Preserve existing parse path and parse-error exit code for empty invocation.
5. Run targeted tests, fmt, and clippy gates.

## Affected Modules
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/main.rs`
- `crates/kamn-cli/tests/main_contract.rs`
- `specs/6119/spec.md`
- `specs/6119/plan.md`
- `specs/6119/tasks.md`

## Risks / Mitigations
- Risk: help detection could shadow valid passthrough args.
  Mitigation: limit detection to explicit `--help`, `-h`, or `help` token and keep non-help parse path unchanged.
- Risk: output drift breaks downstream checks.
  Mitigation: enforce deterministic marker assertions in integration tests.

## Interfaces / Contracts
- New public helpers in `kamn-cli`:
  - `render_help_text() -> &'static str`
  - `is_help_request<I, S>(args: I) -> bool`
- Runtime contract: help requests short-circuit parse/dispatch and return code `0`.
