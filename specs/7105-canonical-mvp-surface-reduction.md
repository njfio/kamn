# Issue 7105: Canonical MVP Surface Reduction

## Objective

Make the proven three-agent transaction the unambiguous KAMN product front
door, prevent non-authoritative claims from appearing in canonical `GO`, and
remove a bounded set of already-superseded repository entrypoints without
losing runtime behavior.

## Inputs And Outputs

Inputs:

- Root README, architecture index, evaluator runbook, Make targets, and their
  existing contract tests.
- Canonical `demo-agent-transaction` report generation and `verify-mvp-demo`.
- Manifest-backed superseded-script inventory and replacement evidence.
- Existing behavior, integration, live, docs-contract, and compatibility tests.

Outputs:

- One clearly named canonical Pi/devnet command and one subordinate local-only
  proof command.
- An architecture map that classifies canonical, compatibility, local-only,
  dry-run, placeholder, and roadmap surfaces.
- Canonical `GO` reports containing no `placeholder` or `dry-run` claims,
  whether required or optional.
- Twenty-five removed superseded symlink entrypoints with manifest-runner
  replacement coverage retained.
- One fewer historical docs-marker test binary and a documented test taxonomy.

## Boundaries And Non-Goals

- Reuse the existing README, architecture index, evaluator runbook, report
  verifier, manifest lane runner, inventory fixtures, and test contracts.
- Do not add a new crate, dependency, orchestration layer, product feature, or
  documentation taxonomy file.
- Preserve `make demo-mvp` as an explicitly local-only compatibility proof; do
  not let it imply devnet settlement or replace `make demo-agent-transaction`.
- Preserve generic run-contract placeholder orchestration as a classified
  compatibility surface; it must remain disconnected from canonical MVP `GO`.
- Do not delete historical evidence documents, active inventory fixtures,
  behavior tests, integration tests, live tests, or replacement runners.
- Do not archive broad spec/doc families in this issue.
- The approved cleanup is limited to entries already named by
  `superseded_script_deletion_manifest.json` plus the non-authoritative
  historical rehearsal marker test identified below.

## Failure Modes

- README or Make help presents `make demo-mvp` as equally canonical with the
  Pi/devnet transaction command.
- Architecture navigation lists surfaces without claim or authority classes.
- A canonical `GO` report includes an optional `placeholder` or `dry-run` claim.
- Placeholder run-contract output is mistaken for transaction proof.
- A deleted symlink has no manifest-runner replacement evidence.
- Deletion leaves stale executable references or breaks command dispatch.
- Removing a docs-marker test also removes behavior coverage.
- Surface metrics claim source LOC reduction when only symlink entrypoints were
  removed.

## Error Semantics

- Any `placeholder` claim in a `GO` report fails verification with
  `required MVP claim cannot be placeholder` or an equally stable explicit
  placeholder reason.
- Any `dry-run` claim in a `GO` report fails verification with
  `required MVP claim cannot be dry-run` or an equally stable explicit dry-run
  reason.
- Missing replacement evidence, stale references, or manifest drift is a hard
  `NO-GO`; no deleted wrapper is silently restored as a fallback.
- Existing structured verifier and shell-policy errors remain fail-closed.

## Acceptance Criteria

- [ ] README leads with `make demo-agent-transaction` and explicitly classifies
  `make demo-mvp` as local-only compatibility proof.
- [ ] Make help uses the same canonical/local-only command taxonomy.
- [ ] Architecture index classifies canonical runtime, compatibility,
  local-only, dry-run, placeholder, and roadmap surfaces.
- [ ] Evaluator runbook uses the same command and claim language.
- [ ] Canonical verifier rejects optional as well as required `placeholder` and
  `dry-run` claims in `GO` reports.
- [ ] Generic placeholder orchestration remains outside the canonical command
  and verifier success path.
- [ ] Test taxonomy distinguishes behavior, integration, live, docs-contract,
  and legacy compatibility tests.
- [ ] Twenty-five manifest-declared superseded symlinks are removed while their
  inventory and manifest-runner replacement evidence remain green.
- [ ] The 68-line historical evaluator rehearsal marker test is removed after
  canonical runtime/verifier contracts remain green.
- [ ] Measured reduction reports 26 files/entrypoints removed, 25 symlinks
  removed, 68 Rust test LOC removed, and zero claimed shell source LOC removed.
- [ ] Formatting, strict clippy, `make check`, `make test`, `make pre-push`,
  canonical demo, and canonical verifier pass.

## Files To Touch

- `README.md`
- `Makefile`
- `docs/architecture/README.md`
- `docs/validation/mvp-evaluator-demo.md`
- `crates/kamn-e2e-harness/src/mvp_demo/verify.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/readme_mvp_front_door_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_demo_runbook_contract.rs`
- Delete `crates/kamn-e2e-harness/tests/mvp_evaluator_rehearsal_docs_contract.rs`.
- Delete only the twenty-five present symlinks already listed in
  `fixtures/ci/superseded_script_deletion_manifest.json`: four under
  `scripts/deploy/`, three under `scripts/governance/`, and eighteen under
  `scripts/kolme/`.
- Keep the deletion manifest, inventory baseline, replacement manifests,
  dispatchers, and stale-reference checkers unchanged unless a RED contract
  proves a narrow correction is necessary.

## Test Plan

### RED

1. Require README and runbook to name `make demo-agent-transaction` as the sole
   canonical settlement path and classify `make demo-mvp` as local-only.
2. Require the architecture index to expose all six surface classes and a test
   taxonomy.
3. Append optional `placeholder` and `dry-run` claims to otherwise valid `GO`
   reports; require verifier rejection.
4. Require all twenty-five approved superseded paths to be absent while each
   path remains present in the deletion inventory with replacement evidence.
5. Require stale-reference and manifest-lane dispatch checks to remain green.

### GREEN

- Align README, Make help, runbook, and architecture index wording.
- Reject non-authoritative labels for every claim in canonical `GO` reports.
- Remove only the approved superseded symlinks after replacement checks pass.

### REFACTOR

- Reuse existing claim-label constants and verifier iteration.
- Keep functions below 25 lines and touched Rust files below 200 lines.
- Remove the historical rehearsal marker test only after proving its assertions
  do not guard runtime behavior.
- Recount files, symlinks, and physical LOC; do not count target-file LOC once
  per removed symlink.

### INTEGRATION

```bash
cargo test -p kamn-e2e-harness --test mvp_demo_claim_contract
cargo test -p kamn-e2e-harness --test readme_mvp_front_door_contract
cargo test -p kamn-e2e-harness --test mvp_evaluator_demo_runbook_contract
bash scripts/ci/test_check_superseded_script_deletion_manifest.sh
bash scripts/ci/test_check_stale_script_references.sh
bash scripts/ci/test_ci_tools_command_surface_contract.sh
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
make test
make pre-push
make demo-agent-transaction
cargo run -p kamn-e2e-harness -- verify-mvp-demo \
  --report .kamn/demo/latest/proof/report.json
```

## Rollback

- Preserve spec, RED, GREEN, reduction, refactor, and integration commits.
- Restore removed symlinks only from their recorded targets if a replacement
  parity check fails; do not recreate copied wrapper bodies.
- Revert claim-label policy independently if it rejects a documented legitimate
  canonical claim class.
- Never delete the manifest, replacement evidence, or live proof artifacts used
  to audit a completed devnet transaction.
