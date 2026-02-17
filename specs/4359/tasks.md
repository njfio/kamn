# Tasks: #4359 Deployment Safety Gate Convergence

T1 (RED): Add failing conformance tests for rotation taxonomy drift and ci/local boundary drift.
- Tier: Functional, Conformance, Regression
- Files: `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`

T2 (GREEN): Implement milestone aggregate checker enforcement for rotation taxonomy + boundary markers.
- Tier: Unit/Functional (script contract)
- Files: `scripts/deploy/gonogo_evidence_contract.py`

T3 (Parity): Update deploy contract-lane fixtures to include new required marker fields where needed.
- Tier: Integration
- Files: `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`

T4 (Docs): Document deployment safety convergence markers and boundaries.
- Tier: Regression/docs
- Files: `docs/ci/strategy.md`

T5 (Verify): Run targeted deploy script tests, then repo gates.
- Commands: deploy tests + `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test` + `cargo mutants --in-diff` (best-effort)
