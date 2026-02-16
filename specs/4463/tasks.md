# Tasks: Issue #4463

Status: In Progress
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
