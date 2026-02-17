# Plan — Issue #4826

## Approach

- Add shared helper primitives first (`common.sh` + executable helper command) so migration is a mechanical line replacement.
- Use scripted transformation for ROOT_DIR-based JSON heredoc writes:
  - `cat ... <<JSON` -> `bash "$ROOT_DIR/scripts/lib/write_json_file.sh" ... <<JSON`
  - only when heredoc content is JSON-like (`{ ... : ... }`).
- Validate with explicit migration contract + full CI regression suite.

## Affected Modules

- `scripts/lib/common.sh`
- `scripts/lib/write_json_file.sh`
- `scripts/lib/test_json_write_helper_migration_contract.sh`
- 89 migrated shell scripts in:
  - `scripts/runtime/`
  - `scripts/kolme/`
  - `scripts/ci/`
  - `scripts/sdk/`
  - `scripts/deploy/`
  - `scripts/channel/`
  - `scripts/message/`
  - `scripts/did/`
  - `scripts/bridge/`

## Risks / Mitigations

- Risk: migration script accidentally rewrites non-JSON heredocs.
  Mitigation: transform only `cat` + redirect + heredocs whose body begins with `{` and includes `:`.
- Risk: runtime behavior drift in CI contract scripts.
  Mitigation: run `bash -n` on changed shell files and full `bash scripts/ci/test_ci_tools.sh`.
- Risk: partial migration leaves hidden legacy pockets.
  Mitigation: migration contract test enforces zero remaining ROOT_DIR-based manual `cat` JSON heredoc writers.

## Interfaces / Contracts

- New helper command interface:
  - `bash scripts/lib/write_json_file.sh <output-json-path>` (JSON content from stdin).
- Existing script command surfaces and key=value outputs remain unchanged.

## ADR

- Not required; no dependency/protocol/architecture change.
