# Tasks: Issue #4463

Status: Completed
Issue: #4463

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add incident readiness stale/mismatch/tamper RED tests to
  `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- Add docs-contract RED assertions to `crates/kamn-core/tests/incident_readiness_docs.rs`.
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `cargo test -p kamn-core --test incident_readiness_docs`
- Expect RED before implementation/docs updates.

T2 (GREEN, Implementation):
- Implement deterministic incident-readiness gate builder/checker convergence in
  `scripts/deploy/gonogo_evidence_contract.py`.
- Wire contract lane scenario in `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`.

T3 (GREEN, Docs):
- Update `docs/ops/incident-readiness.md` with incident-readiness go/no-go gate schema and
  deterministic reason taxonomy markers.

T4 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `cargo test -p kamn-core --test incident_readiness_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --incident-readiness-report-file ... --incident-readiness-max-age-seconds 1800`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Failed with:
      - `expected go/no-go evidence shared contract module to exercise incident-readiness gate arguments`
  - `cargo test -p kamn-core --test incident_readiness_docs`
    - Failed with:
      - `assertion failed: DOC.contains("## Go/No-Go Incident Readiness Bundle Convergence Gate (Issue #4470)")`
      - `assertion failed: DOC.contains("Mismatch and tamper failure cases")`

- GREEN command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`
  - `cargo test -p kamn-core --test incident_readiness_docs`
    - Passed: `2 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Incident-readiness gate now fails closed on missing/invalid/stale/non-GO/tampered incident
    readiness bundles with deterministic reason taxonomy.
  - Checker enforces deterministic incident-readiness gate convergence and rejects tampered payload
    markers.
  - Incident readiness docs now carry explicit go/no-go incident gate contracts and are protected by
    docs tests.
