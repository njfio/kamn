# R42 API and Test-Surface Audit

As of: 2026-02-19  
Issue: #5181  
Milestone: `specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md`

public_api_surface_audit_version=kamn.r42.public-api-surface-audit.v1
shell_rust_test_surface_audit_version=kamn.r42.shell-rust-test-surface-audit.v1
api_surface_ratchet_recommendation_status=proposed
test_surface_ratchet_recommendation_status=proposed
follow_up_issue_api_surface_ratchet=#5188
follow_up_issue_test_surface_migration=#5189
audit_follow_up_issue_count=2

## Method

- Public API inventory command:
  - `rg -n -P '^pub(?!\\(crate\\))\\s+(fn|struct|enum|trait|type|const|static|mod|use)\\b' crates/kamn-core/src -g '*.rs'`
- Test-surface inventory commands:
  - `find scripts -type f -name 'test_*.sh'`
  - `find crates -type f -path '*/tests/*.rs'`

## Findings: Public API Surface

- `kamn_core_public_module_count=82` (from `crates/kamn-core/src/lib.rs`)
- `kamn_core_public_item_count=1266` (non-`pub(crate)` exports across `crates/kamn-core/src/**/*.rs`)
- `kamn_core_pub_item_count_lib_rs=164` (`pub mod` + `pub use` declarations in `crates/kamn-core/src/lib.rs`)

Top non-root files by public-item count:

| Rank | File | Public Items |
|---|---|---:|
| 1 | `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs` | 38 |
| 2 | `crates/kamn-core/src/data_layer_m2_gateway_access.rs` | 34 |
| 3 | `crates/kamn-core/src/data_layer_m4_escrow_integration.rs` | 31 |
| 4 | `crates/kamn-core/src/zk_message_proofs.rs` | 30 |
| 5 | `crates/kamn-core/src/data_layer_m5_vector_integration.rs` | 27 |
| 6 | `crates/kamn-core/src/data_layer_m10_partition_archival.rs` | 26 |
| 7 | `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs` | 25 |
| 8 | `crates/kamn-core/src/data_layer_m1.rs` | 25 |
| 9 | `crates/kamn-core/src/block_pipeline/block_pipeline_support.rs` | 23 |
| 10 | `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs` | 22 |

Candidate `pub(crate)` hardening targets (low-risk, phased, compatibility-reviewed):

- `crates/kamn-core/src/audit_exports.rs` (internal governance helper semantics)
- `crates/kamn-core/src/smoke.rs` (deterministic smoke simulation contract surface)
- `crates/kamn-core/src/snapshot_migration.rs` (migration parity utility surface)
- `crates/kamn-core/src/performance_targets.rs` (benchmark threshold helpers)
- `crates/kamn-core/src/operator_dashboard_ui.rs` (presentation-projection internals)

These candidates should be moved behind curated façade exports in `crates/kamn-core/src/lib.rs`, with compatibility checks in `kamn-node` and `kamn-sdk`.

## Findings: Shell vs Rust Test Surface

- `shell_test_files=529`
- `rust_test_files=294`
- `shell_to_rust_test_file_ratio=1.7993`

Top shell test lanes by file count:

| Lane | Shell Test Files |
|---|---:|
| `scripts/ci` | 118 |
| `scripts/runtime` | 116 |
| `scripts/kolme` | 88 |
| `scripts/sdk` | 35 |
| `scripts/deploy` | 18 |

`cargo test`-invoking shell tests: `10` files (`scripts/ci` + `scripts/kolme`, `4477` LOC total).  
Low-risk wave-1 migration candidates (small wrappers first):

- `scripts/ci/test_makefile_command_surface_contract.sh` (61 LOC)
- `scripts/ci/test_makefile_execution_contract.sh` (35 LOC)
- `scripts/ci/test_run_cargo_test_with_quarantine.sh` (47 LOC)

## Ratchet Recommendations

### API Surface Ratchet (low-risk)

- Add a machine-readable public API report artifact generated from Rust source scan.
- Introduce policy thresholds:
  - `warn` when total public items increase relative to baseline.
  - `fail` when growth exceeds a configured small budget without waiver.
- Keep implementation Rust-first (no new shell wrappers).

Implementation task: #5188

### Test-Surface Ratio Ratchet (low-risk)

- Track `shell_test_files`, `rust_test_files`, and `shell_to_rust_test_file_ratio` in CI artifacts.
- Fail closed on net shell test growth unless paired with documented Rust migration/deletions.
- Require waiver link for temporary regression.

Implementation task: #5189

## Recommended Execution Order

1. Implement API report + ratchet policy (`#5188`).
2. Execute shell-to-rust migration wave-1 (`#5189`).
3. Refresh baselines and update threshold fixtures after each merge wave.
