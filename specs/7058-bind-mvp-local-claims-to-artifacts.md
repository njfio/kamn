# 7058 Bind MVP Local Claims To Artifacts

## Objective

Make the command-level MVP verifier prove that required local MVP claims are
backed by concrete proof artifacts. A report should not pass because its claim
matrix has the right labels while local proof files are missing, static-only, or
tampered.

## Inputs/Outputs

Inputs:
- `.kamn/demo/<run-id>/proof/report.json`.
- Artifact paths listed in the report's `artifacts` object.
- Local proof files written by the MVP demo:
  - `localhost-signed-demo.json`
  - `localhost-signed-demo-output.txt`
  - `service-api-vertical-slice-output.txt`
  - `service-api-websocket-output.txt`
  - `audit-export.json`
  - state, relay, and websocket event files under the run directory.

Outputs:
- Command-level verifier failures for missing or malformed local proof
  artifacts.
- Local MVP proof artifacts that explicitly bind to the real localhost signed
  demo and service API proof commands.
- A report that still verifies for local-only and devnet-backed demo runs when
  all artifacts are valid.

## Boundaries/Non-goals

- Do not redesign the KAMN runtime or service API.
- Do not change Solana devnet settlement behavior or three-agent transcript
  semantics.
- Do not add dependencies.
- Do not count dry-run, placeholder, in-memory-only settlement, or fake
  value-movement output as MVP success.
- Do not close parent tracker `#7020` until this binding gap is verified.

## Failure Modes

- A report references a missing local proof file.
- `localhost-signed-demo.json` lacks the expected schema or pass marker.
- `localhost-signed-demo-output.txt` lacks the localhost signed demo success
  marker.
- The service API vertical slice log lacks its exact test marker or successful
  test result.
- The service API websocket log lacks its exact test marker or successful test
  result.
- Local state, relay, websocket event, or audit artifact content does not show
  that it was bound to the current localhost signed and service API proof
  outputs.
- Local-only reports are incorrectly forced to include settlement or
  three-agent devnet artifacts.
- Devnet-backed reports regress existing devnet settlement or three-agent
  transcript validation.

## Acceptance Criteria

- [ ] `verify-mvp-demo` rejects reports whose required local artifact files are
  missing.
- [ ] `verify-mvp-demo` rejects local artifacts that lack the expected
  runtime/auth/signed-flow/state/relay/websocket/audit proof markers.
- [ ] `make demo-mvp` writes local artifacts whose contents are derived from or
  explicitly bound to the real localhost signed demo and service API proof
  outputs.
- [ ] Local-only reports remain valid when all required local artifacts are
  present and settlement/three-agent claims are absent.
- [ ] Devnet-backed reports continue to validate existing devnet settlement and
  three-agent transcript boundaries.
- [ ] No dry-run, placeholder, in-memory-only settlement, or fake
  value-movement path is counted as MVP success.

## Files To Touch

- `crates/kamn-e2e-harness/src/mvp_demo/local_artifacts.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/runner.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/mod.rs`
- `crates/kamn-e2e-harness/src/mvp_demo/local_artifact_verify.rs`
- `crates/kamn-e2e-harness/tests/mvp_demo_local_artifact_contract.rs`
- Existing MVP verifier or report tests if integration assertions need updates.

## Error Semantics

- Missing or malformed local artifacts fail closed with explicit `Err(String)`
  messages.
- Command-level verification must read artifact files from report paths. It must
  not silently trust the inline claim matrix.
- Demo generation errors while writing bound local artifacts fail the demo.
- Local-only mode remains local-only; missing devnet evidence must not be
  converted into a fake settlement pass.

## Test Plan

Red:
- Add command-level verifier tests that reject a report with a missing local
  artifact.
- Add command-level verifier tests that reject a tampered local artifact missing
  the service API or localhost proof marker.
- Add a local-only fixture proving that a valid artifact bundle verifies without
  settlement or three-agent claims.

Green:
- Add local artifact verification helpers that parse report artifact paths and
  validate file contents.
- Change local artifact generation so state, relay, websocket, and audit files
  carry explicit bindings to the real proof command outputs.
- Wire local artifact verification into both `demo-mvp` and `verify-mvp-demo`.

Refactor:
- Keep path extraction and marker validation helpers small and single-purpose.
- Reuse existing string marker extraction helpers instead of adding a JSON
  dependency.
- Preserve existing devnet and three-agent validators without semantic changes.

Integration:
- Run the new local artifact contract test.
- Run existing MVP report, claim, three-agent, agent harness, and evaluator
  runbook tests.
- Run `cargo fmt --check`, strict workspace clippy, and `make check`.
- Run `make demo-mvp` and canonical `verify-mvp-demo`.
