# Issue #4137 Plan

- Issue: #4137
- Status: Implemented

## Approach
1. Add a red modularization contract test that expects root-to-submodule split wiring.
2. Extract shared task/escrow property helpers into a dedicated shared module file.
3. Move task-domain tests into `task_domain.rs` and escrow-domain tests into `escrow_domain.rs`.
4. Keep root integration file as module harness only.
5. Add/update testing strategy docs for modularization conventions.
6. Run targeted suites, fmt, clippy, and shell guardrails.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Refactor could accidentally drop tests or alter deterministic configuration.
  - Module-path wiring drift can silently break split architecture.
- Mitigations:
  - Use contract test to pin module declarations and required files.
  - Preserve existing test function names and rerun full target binary.

## Interface Contract
- Test organization changes only.
- No production API behavior change.

## ADR
- Not required (test-surface organization refactor).
