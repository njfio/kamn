# Plan: #5666 Enable cargo-mutants In-Diff Gate for Portable-Agent Slices

## Approach
1. Record RED evidence for missing `cargo-mutants` command.
2. Install `cargo-mutants` in the execution environment and capture GREEN evidence.
3. Add mutation-gate guidance to `docs/ci/strategy.md`:
   - install command
   - in-diff invocation command
   - fallback behavior when tooling is unavailable
4. Re-run the in-diff command to verify invocation remains valid after docs updates.

## Affected Modules
- `docs/ci/strategy.md`
- `specs/5666/spec.md`
- `specs/5666/tasks.md`

## Risks and Mitigations
- Risk: `cargo mutants --in-diff` can be expensive on large diffs.
- Mitigation: document `--list` usage for bounded validation and keep scope in-diff.

- Risk: local environments may not allow tooling install.
- Mitigation: document deterministic fallback evidence + follow-up issue requirement.

## Interfaces / Contracts
- Mutation tier command surface for this task:
  - `cargo mutants --version`
  - `cargo mutants --in-diff --list`

## ADR
- Not required. No dependency is added to workspace manifests and no protocol/API contract changes.
