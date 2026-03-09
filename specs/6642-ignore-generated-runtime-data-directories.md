# 6642 Ignore Generated Runtime Data Directories

## Objective
Declare generated runtime storage directories as ignored workspace state so local runtime data does not pollute `git status` or get treated like candidate source changes during workspace hygiene.

## Inputs/Outputs
- Inputs:
  - `.gitignore`
  - current runtime storage defaults under `./data/...`
  - existing workspace gitignore contract test patterns
- Outputs:
  - explicit `.gitignore` markers for generated runtime data directories
  - a deterministic contract test that fails if those markers are removed or renamed

## Boundaries/Non-goals
- Do not change runtime storage defaults or profile wiring
- Do not ignore broader path families than the current generated runtime data directories
- Do not modify CI, workflows, or shell tooling
- Do not delete tracked files or tracked directories

## Failure modes
- runtime data directories remain unignored and reappear in `git status`
- ignore markers drift or are removed without a failing contract
- `.gitignore` broadens far enough to hide tracked source unintentionally

## Acceptance criteria
- [ ] `.gitignore` contains deterministic markers for `data/` and `crates/kamn-node/data/`
- [ ] a contract test fails if either required ignore marker is absent
- [ ] the contract only validates the intended runtime-data markers and does not depend on ambient workspace state

## Files to touch
- `.gitignore`
- `crates/kamn-core/tests/workspace_gitignore_runtime_data_policy_contract.rs`
- `specs/6642-ignore-generated-runtime-data-directories.md`

## Error semantics
- missing ignore markers are hard failures in the contract test
- file-read failures in the contract test are hard failures
- no fallback or auto-generation of ignore markers

## Test plan
1. Add a contract test for required runtime-data ignore markers and verify it fails before implementation
2. Add the `.gitignore` markers
3. Re-run the targeted contract tests and require green
4. Confirm `git status --short` remains clean after the change
