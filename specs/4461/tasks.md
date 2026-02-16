# Tasks: Issue #4461

Status: Completed
Issue: #4461

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add audit-integrity gate assertions and tamper/unstable-output drills in
  `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- Add docs-contract assertions for audit references in release and ops docs tests.
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs`
- Expect RED before implementation/docs updates.

T2 (GREEN, Implementation):
- Implement deterministic audit-integrity gate build + checker convergence in
  `scripts/deploy/gonogo_evidence_contract.py`.
- Keep reason taxonomy and reason-code outputs deterministic.

T3 (GREEN, Docs):
- Update `docs/foundation/release-gonogo-checklist.md` and `docs/ops/configuration.md` with audit
  integrity contract markers/commands.

T4 (Verify):
- Run:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Failed with:
      - `gonogo_evidence_contract.py: error: unrecognized arguments: --audit-integrity-report-file ... --audit-integrity-max-age-seconds 1800`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_audit_integrity_convergence_gate -- --exact`
    - Failed with:
      - `assertion failed: CHECKLIST.contains("## Audit-Trail Integrity/Tamper Convergence Gate (Issue #4466)")`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_audit_integrity_tamper_controls -- --exact`
    - Failed with:
      - `assertion failed: DOC.contains("## Audit Integrity Go/No-Go Policy Controls (Issue #4465)")`

- GREEN command/output:
  - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
    - Passed: `go/no-go evidence bundle tests passed.`
  - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - Passed: `go/no-go evidence contract lane script tests passed.`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
    - Passed: `67 passed; 0 failed`
  - `cargo test -p kamn-core --test service_api_ops_configuration_docs`
    - Passed: `2 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Go/no-go audit integrity gate now fails closed on missing/invalid/stale/non-pass/tampered
    audit policy artifacts with deterministic reason taxonomy.
  - Checker enforces deterministic audit gate convergence and rejects tampered payload markers.
  - Ops/release docs now carry explicit audit gate marker contracts and are protected by docs tests.
