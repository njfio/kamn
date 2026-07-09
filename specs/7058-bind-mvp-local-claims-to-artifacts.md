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

- [x] `verify-mvp-demo` rejects reports whose required local artifact files are
  missing.
- [x] `verify-mvp-demo` rejects local artifacts that lack the expected
  runtime/auth/signed-flow/state/relay/websocket/audit proof markers.
- [x] `make demo-mvp` writes local artifacts whose contents are derived from or
  explicitly bound to the real localhost signed demo and service API proof
  outputs.
- [x] Local-only reports remain valid when all required local artifacts are
  present and settlement/three-agent claims are absent.
- [x] Devnet-backed reports continue to validate existing devnet settlement and
  three-agent transcript boundaries.
- [x] No dry-run, placeholder, in-memory-only settlement, or fake
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

## Completion Evidence

- Red evidence captured:
  `mvp_demo_local_artifact_contract` failed 2 expected cases because
  command-level verification accepted reports with missing local artifacts and
  tampered service API proof logs.
- Targeted contracts passed:
  `mvp_demo_local_artifact_contract`,
  `mvp_demo_three_agent_transcript_contract`,
  `mvp_demo_agent_harness_claim_contract`, `mvp_demo_claim_contract`,
  `mvp_demo_three_agent_claim_contract`, and
  `mvp_evaluator_demo_runbook_contract`.
- `cargo fmt --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed with `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1
  CARGO_INCREMENTAL=0`.
- `make check` passed with the same cargo environment.
- Local `make demo-mvp` passed with run `run-97346-1783566389296`; canonical
  `verify-mvp-demo` returned `{"status":"PASS"}`.
- The local run's generated artifacts now include explicit bindings:
  `runtime-state.json` has `source:"localhost-signed-demo"`,
  `relay-projection.json` and `audit-export.json` have
  `source:"service-api-vertical-slice"`, and `websocket-events.json` has
  `source:"service-api-websocket"`.
- Devnet-required `make demo-mvp` passed with run
  `run-17119-1783566532057`, finalized Solana devnet signature
  `2uQqBgSdGdwfowsQtNu7NEyv6h2ghPB84oVPitTSTa2NT76FNAP9P9S337Ft2oSc6rD5gEnFz8do7U6U7XWcdVHE`,
  `1000000` lamports, and canonical verifier `PASS`.
- The devnet run retained `devnet-backed` settlement and three-agent claims;
  the transcript linked the same signature, preserved participant-private
  Agent A/B views, kept Agent C restricted-public, and kept raw private payloads
  redacted.

## Deviations

- None.
