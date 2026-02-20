# Issue #5320 Plan

## Approach
1. Apply rustfmt-compliant formatting for the failing assertion block in `crates/kamn-core/tests/ci_strategy_docs.rs`.
2. Run `cargo fmt --all --check` and the targeted docs test.
3. Open and merge an expedited hotfix PR.

## Affected Modules
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/5320/{spec,plan,tasks}.md`

## Risks and Mitigations
- Risk: accidental behavior changes while touching the test file.
  - Mitigation: formatting-only patch + targeted test execution.

## Interfaces and Contracts
- No interface or behavioral contract changes; formatting conformance only.
