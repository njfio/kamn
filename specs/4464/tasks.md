# Tasks: Issue #4464

Status: Completed
Issue: #4464

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add incident go/no-go boundary RED tests in:
  - `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- Add docs-contract RED assertions in:
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
- Run:
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs`
- Expect RED before implementation/docs updates.

T2 (GREEN, Implementation):
- Implement incident go/no-go CI smoke/local-heavy boundary governance in deploy lanes.
- Emit deterministic incident boundary taxonomy/version/reason markers.

T3 (GREEN, Docs):
- Update `docs/ci/strategy.md` with incident go/no-go boundary governance matrix.

T4 (Verify):
- Run:
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Failed with:
      - `expected go/no-go evidence contract lane to emit incident boundary reason taxonomy status marker`
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --incident-readiness-report-file ... --incident-readiness-max-age-seconds 1800`
  - `cargo test -p kamn-core --test ci_strategy_docs`
    - Failed with:
      - `assertion failed: DOC.contains("Incident go/no-go convergence and boundary governance")`
      - `assertion failed: DOC.contains("incident_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-incident-boundary-reason-taxonomy.v1")`

- GREEN command/output:
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `cargo test -p kamn-core --test ci_strategy_docs`
    - Passed: `26 passed; 0 failed`
  - `bash scripts/runtime/test_run_go_no_go_gate_lane.sh`
    - Passed: `go/no-go gate lane script tests passed.`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Incident go/no-go CI smoke boundary now fails closed on overflow with deterministic
    `incident_gonogo_ci_smoke_seconds_exceeded`.
  - Incident deep-lane local-heavy path now requires explicit opt-in and fails closed with
    deterministic local-heavy boundary reason codes.
  - CI strategy docs now pin incident boundary taxonomy and ci/local governance markers with docs
    contract tests.
