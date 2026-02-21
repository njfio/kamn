# Issue #5460 Plan - Shell-to-Rust Lane Migration

## Approach
1. RED first: update contract checks to require Rust harness command surface and verify failure before implementation.
2. Add Rust binary harness under `crates/kamn-core/src/bin/` that emits deterministic cross-store lane markers.
3. Replace go/no-go registry cross-store command to invoke Rust binary via `cargo run`.
4. Generalize command availability checks in go/no-go lane contract to support both shell-script paths and executable-in-PATH commands.
5. Remove superseded shell wrapper and rerun focused regression suites.

## Affected Modules
- `crates/kamn-core/src/bin/cross_store_replay_consistency_contract_lane.rs` (new)
- `scripts/runtime/go_no_go_gate_lane_contract.py`
- `scripts/runtime/test_run_go_no_go_gate_lane.sh`
- `scripts/runtime/release_evidence_manifest.json`
- `scripts/runtime/validate_cross_store_replay_consistency_contract_lane.sh` (delete)
- `specs/5460/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: go/no-go executable guard logic breaks existing script-based artifact commands.
  - Mitigation: preserve script-path check behavior and add PATH-based fallback for non-path executables.
- Risk: marker drift from harness migration.
  - Mitigation: keep exact success marker string unchanged.

## Interfaces / Contracts
- Harness command surface:
  - `cargo run -p kamn-core --bin cross_store_replay_consistency_contract_lane --`
- Required success marker:
  - `cross_store_replay_consistency_policy_status=verified`

## Validation Strategy
- RED:
  - `bash scripts/runtime/test_run_go_no_go_gate_lane.sh` after command-surface expectation update.
- GREEN/REGRESSION:
  - `bash scripts/runtime/test_run_go_no_go_gate_lane.sh`
  - `cargo test -p kamn-core --test release_gonogo_checklist_docs -- --nocapture`
  - `cargo fmt --check`
