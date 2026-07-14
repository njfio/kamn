# Issue 7121: Publish MVP Planning Artifacts

## Objective

Publish the secret-safe historical brainstorm and plans referenced by the MVP
tracker, and make Python packaging outputs resolve to an intentional clean
working-tree policy.

## Inputs And Outputs

Inputs:

- Three local historical documents referenced by tracker `#7088`.
- Root Python packaging metadata and ignore policy.
- Existing evaluator-facing MVP claim language and Rust docs contracts.

Outputs:

- Publicly resolvable historical brainstorm and plan paths.
- Explicit historical status and bounded claim language in each artifact.
- A tracked Python lockfile and ignored generated egg metadata.
- A fail-closed Rust contract for artifact presence, references, status, and
  packaging hygiene.

## Boundaries And Non-Goals

- Preserve the documents as dated decision artifacts; do not rewrite them as
  current implementation or release claims.
- Do not publish secrets, generated credentials, local proof bundles, or real
  private key paths.
- Do not change runtime, service API, Pi/MCP, settlement, verifier, protocol,
  dependency, or release behavior.
- Do not broaden the root README or evaluator runbook beyond any minimal link
  needed to make the historical lineage discoverable.
- Do not add shell or Python executable code.

## Cleanup Plan

1. Characterize references, sensitive markers, package outputs, and existing
   documentation contracts before editing.
2. Add RED contracts that require the missing tracked artifacts and explicit
   historical/bounded markers.
3. Publish the reviewed documents, track the lockfile, and ignore only
   generated `*.egg-info/` directories.
4. Consolidate contract helpers and verify no duplicate policy text is needed.
5. Run targeted and repository-wide documentation, format, lint, and test
   gates; verify a clean clone does not inherit local generated metadata.

## Failure Modes

- A required historical artifact remains absent from git.
- A tracker reference resolves to a missing path.
- A historical plan is presented as a current shipped or production claim.
- An artifact contains a real absolute user path, credential, key, token, or
  generated proof material.
- `kamn_sdk.egg-info/` reappears as unexplained untracked repository state.
- The Python lockfile remains unclassified or is ignored as generated output.
- Documentation changes weaken MVP local/devnet/roadmap boundaries.

## Error Semantics

- Missing paths, required historical markers, or tracker-reference markers
  fail the Rust contract with the exact missing marker or path.
- Forbidden secret/path patterns fail closed and identify only the pattern and
  document, never secret content.
- Packaging policy drift fails when `*.egg-info/` is not ignored or `uv.lock`
  is not tracked.

## Acceptance Criteria

- [x] The three tracker-referenced documents are tracked and publicly
  resolvable at their existing paths.
- [x] Each document identifies itself as a historical decision artifact and
  does not claim production or mainnet readiness.
- [x] No document contains a real private path, secret, credential, key
  material, `.kamn` run payload, or generated proof artifact.
- [x] `uv.lock` is tracked as reproducible Python project metadata.
- [x] `*.egg-info/` is ignored as generated Python build metadata.
- [x] A bounded Rust docs contract fails on missing paths, broken lineage,
  unsafe claim markers, or packaging-policy drift.
- [x] Existing README/runbook claim contracts remain green.
- [x] The intentional working tree is clean after publication.

## Files To Touch

- `specs/7121-publish-mvp-planning-artifacts.md`
- `docs/brainstorms/2026-06-26-kamn-forward-strategy-requirements.md`
- `docs/plans/2026-06-26-001-kamn-mvp-demo-readiness-plan.md`
- `docs/plans/2026-07-10-001-kamn-agent-transaction-rail-mega-plan.md`
- `.gitignore`
- `uv.lock`
- `crates/kamn-e2e-harness/tests/mvp_planning_artifact_contract.rs`

## Test Plan

### RED

1. Require all three historical paths to be tracked and readable.
2. Require explicit historical-artifact and bounded MVP markers.
3. Reject real absolute user paths and credential/key material patterns.
4. Require lineage from both plans to the brainstorm and from the later plan
   to the superseded earlier plan.
5. Require `*.egg-info/` ignore policy while preserving tracked `uv.lock`.

### GREEN

- Add the reviewed artifacts with concise historical status notes.
- Track `uv.lock` and add the standard generated metadata ignore rule.

### REFACTOR

- Keep the test file below 200 lines and functions below 25 lines.
- Reuse small table-driven helpers for path, marker, and forbidden-pattern
  checks.
- Remove duplicated explanatory text when one shared marker is sufficient.

### INTEGRATION

```bash
cargo test -p kamn-e2e-harness --test mvp_planning_artifact_contract
cargo test -p kamn-e2e-harness --test readme_mvp_front_door_contract
cargo test -p kamn-e2e-harness --test mvp_evaluator_demo_runbook_contract
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
make test
make pre-push
```

## Completion Evidence

- RED: the new contract failed three cases for missing historical markers,
  bounded claim language, and generated metadata policy.
- GREEN/REFACTOR: the targeted contract passed all four cases; the README and
  evaluator runbook contracts also passed.
- Repository gates: `cargo fmt --check`, strict workspace clippy,
  `make check`, and `make test` passed.
- Local pre-push: workspace tests and critical-path coverage returned `GO`;
  mutation testing caught 10 of 10 mutants with zero misses or timeouts.
- Inventory deviation: the tracked Rust test-file baseline increased from
  1,318 to 1,319 for the new artifact contract. No runtime behavior changed.

## Rollback

- Revert the issue branch as one documentation-hygiene arc if publication
  reveals unsafe content.
- Keep generated metadata ignored even if one historical artifact must be
  withdrawn; replace any withdrawn public reference with an explicit rationale.
- Do not alter runtime or proof semantics to repair a documentation failure.
