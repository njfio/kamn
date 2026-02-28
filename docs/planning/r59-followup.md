# R59 Follow-up

## Issue #6249: kamn-core Wave2 Shim Retirement

### Shim Inventory And Decisions

| Shim module (`kamn-core`) | Extracted owner crate | Decision (R59) | Workspace migration status | Removal target |
|---|---|---|---|---|
| `src/anti_spam.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | `kamn-node` service-api anti-spam path now imports extracted crate directly | R61 |
| `src/fairness_policy.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/quota_policy.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/message_delivery_guards.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/retention_engine.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/watchdog.rs` | `kamn-runtime-guards` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/live_probe_matrix.rs` | `kamn-live-probe-matrix` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |
| `src/cross_chain_receipt.rs` | `kamn-bridges` | Keep temporarily, hard-deprecated | Root `kamn-core` exports now wired directly to extracted crate | R61 |

### Notes

- Shim modules remain only for transitional compatibility and are annotated as deprecated.
- `kamn-core` root exports for the migrated surfaces no longer depend on shim modules; they re-export directly from extracted crates.
- New in-repo imports should target extracted crates directly.
