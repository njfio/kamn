# Tasks: Issue #4434

Status: Completed
Issue: #4434

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add live-go/no-go RED coverage in:
  - `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
  - `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
- Expect deterministic RED failures before implementation/docs updates.

T2 (GREEN, Implementation):
- Implement live go/no-go taxonomy and boundary governance in deploy go/no-go generator/checker and
  lane wrappers.
- Keep existing incident gate surface intact while adding deterministic live gate marker surfaces.

T3 (GREEN, Docs):
- Update:
  - `docs/ci/strategy.md`
  - `docs/foundation/release-gonogo-checklist.md`
- Add live-go/no-go convergence and boundary matrix markers.

T4 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `bash scripts/runtime/test_run_go_no_go_gate_lane.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Failed with:
      - `expected go/no-go evidence contract lane to emit live boundary reason taxonomy status marker`
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `expected deterministic live-go/no-go reason taxonomy marker for milestone aggregate evidence: expected 'kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1', got ''`
  - `cargo test -p kamn-core --test ci_strategy_docs --test release_gonogo_checklist_docs`
    - Failed with:
      - `assertion failed: DOC.contains("Live go/no-go convergence and boundary governance")`
      - `assertion failed: DOC.contains("live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1")`
      - `assertion failed: CHECKLIST.contains("## Live Go/No-Go Evidence Convergence and Boundary Governance Gate (Issue #4434)")`

- GREEN command/output:
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `cargo test -p kamn-core --test ci_strategy_docs --test release_gonogo_checklist_docs`
    - Passed: `97 passed; 0 failed` across both test binaries.
  - `bash scripts/runtime/test_run_go_no_go_gate_lane.sh`
    - Passed: `go/no-go gate lane script tests passed.`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Live go/no-go milestone bundle now emits deterministic taxonomy and reason-code csv markers.
  - Contract/deep lane boundary governance now emits deterministic live boundary markers and
    fail-closed reason codes for CI smoke overflow and local-heavy opt-in/budget violations.
  - CI/docs contract tests pin live-go/no-go convergence and boundary surfaces.
