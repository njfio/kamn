# Spec: Issue 6465 - Centralize workspace dependency versions

## Objective
Remove cross-crate dependency version drift by introducing root-level
`[workspace.dependencies]` entries and consuming them from member crates via
`workspace = true`.

## Inputs/Outputs
- Inputs:
  - Root `Cargo.toml`
  - Member crate manifests under `crates/*/Cargo.toml`
- Outputs:
  - Shared third-party dependency versions defined once in root workspace.
  - Member manifests updated to reference workspace-managed versions.
  - Existing drift for `serde_json`, `zeroize`, and `rustls` resolved.

## Boundaries/Non-goals
- No runtime behavior changes.
- No dependency-major upgrades.
- No CI workflow changes.

## Failure modes
- A crate still pins a divergent direct dependency version.
- Workspace dependency entries are incomplete and break `cargo` resolution.
- Optional/dev dependency wiring changes behavior unexpectedly.

## Acceptance criteria (testable booleans)
- [x] AC-1: Root `Cargo.toml` contains `[workspace.dependencies]` entries for
      shared third-party crates used across multiple workspace members.
- [x] AC-2: Member manifests consume shared versions via `workspace = true`
      where applicable.
- [x] AC-3: No version drift remains for `serde_json`, `zeroize`, and
      `rustls`.
- [x] AC-4: `cargo check --workspace` passes.
- [x] AC-5: `cargo test -p kamn-core --test test_file_size_policy` passes.

## Files to touch
- `specs/6465-centralize-workspace-dependency-versions.md`
- `Cargo.toml`
- `crates/kamn-agent-lib/Cargo.toml`
- `crates/kamn-cli/Cargo.toml`
- `crates/kamn-core/Cargo.toml`
- `crates/kamn-crypto/Cargo.toml`
- `crates/kamn-data-layer/Cargo.toml`
- `crates/kamn-live-probe-matrix/Cargo.toml`
- `crates/kamn-mcp-server/Cargo.toml`
- `crates/kamn-node/Cargo.toml`
- `crates/kamn-sdk/Cargo.toml`
- `crates/kamn-snapshot-journal/Cargo.toml`

## Error semantics
- Preserve existing compilation and test error behavior.
- Fail fast on manifest-resolution problems via `cargo check --workspace`.

## Test plan
- Red:
  - Add manifest contract checks for drift candidates and confirm pre-change
    drift exists.
- Green:
  - Introduce root `[workspace.dependencies]`.
  - Update member manifests to use `workspace = true`.
- Refactor:
  - Keep manifest entries minimal and consistent.
- Integration:
  - `cargo check --workspace`
  - `cargo test -p kamn-core --test test_file_size_policy`

## Phase 6 integration evidence
- Root workspace now owns shared dependency versions through
  `[workspace.dependencies]`, and member crates consume those versions via
  `workspace = true`.
- Verified commands:
  - `cargo test -p kamn-core --test core_extraction_wave1_contract`
  - `cargo check --workspace`
  - `cargo test -p kamn-core --test test_file_size_policy`

## Deviations
- None.
