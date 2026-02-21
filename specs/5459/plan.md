# Issue #5459 Plan - Integrate Cross-store Replay in Go/No-go Gate

## Approach
1. RED first: run targeted go/no-go lane contract tests to capture current omission/mismatch behavior for cross-store replay required artifact.
2. Add a dedicated executable lane wrapper for cross-store replay consistency checks.
3. Expand go/no-go lane contract (`go_no_go_gate_lane_contract.py`) required artifact registry, manifest required IDs, status projection, and output markers.
4. Update release evidence manifest and related tests/docs expectations.
5. Run focused regression suites for runtime lane contract and docs/checklist marker tests.

## Affected Modules
- `scripts/runtime/go_no_go_gate_lane_contract.py`
- `scripts/runtime/release_evidence_manifest.json`
- `scripts/runtime/validate_cross_store_replay_consistency_contract_lane.sh` (new)
- `scripts/runtime/test_run_go_no_go_gate_lane.sh`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `docs/foundation/release-gonogo-checklist.md` (if marker list changes)
- `specs/milestones/r28-1-cross-store-replay-production-go-no-go-integration/index.md`
- `specs/5459/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: reason-code CSV ordering drift breaks deterministic contract tests.
  - Mitigation: keep existing normalized ordering and append only deterministic new markers where required.
- Risk: run-mode lane wrapper output/marker mismatch.
  - Mitigation: use explicit required marker assertion in wrapper and contract tests.
- Risk: shell-surface growth.
  - Mitigation: thin wrapper script and report Shell-Surface DoD actual deltas.

## Interfaces / Contracts
- Required artifact ID: `cross_store_replay_consistency`
- Expected lane: `runtime.validate_cross_store_replay_consistency_contract_lane`
- Expected success marker: `cross_store_replay_consistency_policy_status=verified`

## Validation Strategy
- RED:
  - `bash scripts/runtime/test_run_go_no_go_gate_lane.sh`
  - targeted docs/checklist tests likely to mismatch until markers updated.
- GREEN:
  - rerun above after implementation.
- REGRESSION:
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs -- --nocapture`
  - `cargo test -p kamn-core --test ci_strategy_docs -- --nocapture`
