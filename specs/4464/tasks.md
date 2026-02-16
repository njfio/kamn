# Tasks: Issue #4464

Status: In Progress
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
