# Issue 7114: Repair Agent Harness Verifier Fixture Paths

## Objective

Restore valid agent-harness and direct Pi evidence verification after #7103 by
making legacy test fixtures use the canonical immutable-run/latest report layout,
without relaxing report substitution or proof-bundle containment checks.

## Inputs And Outputs

Input:

- Valid generated MVP reports with bound actor tool receipts or direct Pi actor
  evidence.
- The canonical `<output>/<run-id>/proof/report.json` and
  `<output>/latest/proof/report.json` path contract.

Output:

- Existing valid evidence tests pass from canonical report paths.
- Foreign-report, escaped-artifact, traversal, and substituted markdown tests
  continue to fail with stable public codes.

## Boundaries And Non-Goals

- Prefer correcting fixture construction over weakening production verifier code.
- Do not permit arbitrary report locations or skip canonical-path validation.
- Do not weaken actor receipt, digest, role, private-view, or report-binding checks.
- Do not change public demo commands, schemas, dependencies, or APIs.

## Failure Modes

- A valid fixture still supplies a noncanonical report path.
- Fixture repair mutates the report or actor evidence under verification.
- Direct Pi evidence for another report begins to pass.
- Path traversal, artifact escape, or markdown substitution becomes accepted.

## Error Semantics

Production verification retains `PROOF_ARTIFACT_PATH_INVALID` for noncanonical
report paths. Fixtures must satisfy the same path contract as real demo output.

## Acceptance Criteria

- [x] Existing actor-tool-receipt success fixture passes.
- [x] Existing direct-Pi-evidence success fixture passes without report mutation.
- [x] Different-report direct evidence remains rejected.
- [x] Settlement-only claim fixtures do not index unclaimed three-agent artifacts.
- [x] Three-agent receipt negative fixtures reach their intended receipt checks
  from a canonical independently verifiable proof bundle.
- [x] Three-agent transcript negative fixtures use canonical generated bundles
  and assert stable public verifier codes.
- [x] Three-agent view negative fixtures preserve digest and receipt bindings
  until the intended view-scope or identity validation fails.
- [x] Independent path-security and actor negative matrices remain green.
- [x] No production verifier containment check is removed or bypassed.
- [x] Formatting, strict clippy, `make check`, and `make test` pass.

## Files To Touch

- Focused fixture helpers under
  `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract*`.
- `crates/kamn-e2e-harness/tests/mvp_demo_claim_contract.rs`, to remove
  contradictory three-agent artifact indexes from settlement-only fixtures.
- `crates/kamn-e2e-harness/tests/mvp_demo_three_agent_claim_contract.rs` and
  focused receipt/transcript fixture support, to preserve claim-specific
  artifact indexes and canonicalize negative-test setup.
- `fixtures/ci/test_file_size_policy_baseline.env`, because the mandatory
  fixture-helper split adds two counted Rust test files to the exact inventory;
  support-only helpers remain excluded by policy.
- This spec only.

## Integration Deviation

The mandatory refactor split canonical settlement construction, generic fixture
I/O, and generated receipt mutation into bounded files below 200 lines. The
receipt helper is under `tests/support` and therefore remains excluded from the
policy inventory. The counted test-file inventory remains 1,319, up from 1,317;
oversized-file counts and all policy thresholds remain unchanged.

## Test Plan

RED is the existing committed 2-of-23 failure:

```text
spec_c18 ... PROOF_ARTIFACT_PATH_INVALID
spec_c22 ... PROOF_ARTIFACT_PATH_INVALID
```

GREEN verification:

```bash
CARGO_TARGET_DIR=target/mvp-demo-proof cargo test -p kamn-e2e-harness \
  --test mvp_demo_agent_harness_claim_contract
CARGO_TARGET_DIR=target/mvp-demo-proof cargo test -p kamn-e2e-harness \
  --test independent_verifier_path_security_contract \
  --test independent_agent_transaction_actor_verifier_contract
cargo fmt --check
CARGO_TARGET_DIR=target/mvp-demo-proof cargo clippy \
  --workspace --all-targets --all-features -- -D warnings
CARGO_TARGET_DIR=target/mvp-demo-proof make check
CARGO_TARGET_DIR=target/mvp-demo-proof make test
```
