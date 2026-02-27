# Plan: Issue #6110

## Approach
1. Add RED regression tests around state IO atomic replacement and temp-artifact cleanup.
2. Implement a shared atomic file-write helper in `state_io.rs` (temp file in target directory + fsync + rename).
3. Route JSON state persistence and relay spool drain truncation through the helper.
4. Preserve sqlite backend logic and existing reason/error prefixes.
5. Run targeted `kamn-node` tests and quality gates.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/state_io.rs`
- `crates/kamn-node/src/service_api_endpoint/tests.rs` (if additional coverage needed)
- `specs/6110/spec.md`
- `specs/6110/plan.md`
- `specs/6110/tasks.md`

## Risks / Mitigations
- Risk: temporary-file naming collision.
  Mitigation: include pid + nanosecond nonce in temp filename and create with `create_new`.
- Risk: temp artifact leak on intermediate failure.
  Mitigation: best-effort cleanup on write/sync/rename errors and explicit regression assertions.

## Interfaces / Contracts
- Internal state IO contract only; no API/wire-format changes.
- Error prefixes remain stable (`service api state file write failed`, `service api relay spool truncate failed`).
