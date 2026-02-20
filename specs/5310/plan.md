# Plan — Issue #5310

Issue: #5310
Spec: `specs/5310/spec.md`

## Approach

1. Migrate a focused set of tiny CI wrappers to symlinks targeting `scripts/lib/exec_dispatch.sh`.
2. Add corresponding registry entries in `scripts/lib/exec_registry.json`.
3. Add `${KAMN_ROOT}` token expansion in `scripts/lib/exec_dispatch.py` for robust `--repo-root` and threshold-path prefixes.
4. Extend `scripts/lib/test_exec_dispatch_registry.sh` to validate token expansion behavior.
5. Run targeted wrapper and dispatcher tests, then shell guardrail checks and baseline-delta measurement.

## Affected Areas

- `scripts/lib/exec_dispatch.py`
- `scripts/lib/exec_registry.json`
- `scripts/lib/test_exec_dispatch_registry.sh`
- Migrated wrapper scripts under `scripts/ci/`

## Risks and Mitigations

- Risk: wrapper argument semantics drift.
  - Mitigation: preserve interpreter/target/prefix semantics in registry entries and run existing wrapper tests.

- Risk: token expansion introduces unexpected substitutions.
  - Mitigation: restrict to explicit `${KAMN_ROOT}` replacement only; add regression assertion.

- Risk: shell LOC reduction is too small.
  - Mitigation: migrate multiple wrappers in this change-set and verify measured deltas vs #4042 baseline.

## Interfaces/Contracts

- Registry contract remains `version = kamn.exec-wrapper-registry.v1`.
- Wrapper dispatch contract remains interpreter+target+args_prefix+passthrough.
- New supported token in `args_prefix`: `${KAMN_ROOT}`.

