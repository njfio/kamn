# Tasks: Issue #4462

Status: Completed
Issue: #4462

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add SLO threshold drift + gate-mismatch acceptance tests in
  `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- Add docs-contract assertions for SLO gate markers in release + observability docs tests.
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo test -p kamn-core --test observability_schema_docs`
- Expect RED before implementation/docs updates.

T2 (GREEN, Implementation):
- Implement deterministic SLO policy gate builder and checker convergence in
  `scripts/deploy/gonogo_evidence_contract.py`.

T3 (GREEN, Docs):
- Update `docs/foundation/release-gonogo-checklist.md` and `docs/observability/schema.md` for SLO
  threshold/gate taxonomy references.

T4 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo test -p kamn-core --test observability_schema_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --slo-policy-report-file ... --slo-policy-max-age-seconds 1800`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_slo_threshold_policy_gate_convergence -- --exact`
    - Failed with:
      - `assertion failed: CHECKLIST.contains("## SLO Threshold/Policy Gate Convergence Gate (Issue #4468)")`
  - `cargo test -p kamn-core --test observability_schema_docs observability_schema_contains_slo_threshold_and_gate_taxonomy_matrix -- --exact`
    - Failed with:
      - `assertion failed: DOC.contains("## SLO Threshold and Gate Reason Taxonomy Matrix (Issue #4462)")`

- GREEN command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
    - Passed: `68 passed; 0 failed`
  - `cargo test -p kamn-core --test observability_schema_docs`
    - Passed: `2 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Go/no-go SLO policy gate now fails closed on missing/invalid/stale/non-pass/tampered SLO policy
    artifacts with deterministic reason taxonomy.
  - Checker enforces deterministic SLO gate convergence and rejects tampered payload markers.
  - Release and observability docs now carry explicit SLO threshold/gate contracts and are protected
    by docs tests.
