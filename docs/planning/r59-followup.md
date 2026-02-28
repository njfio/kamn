# R59 Follow-Up Wave (Issues #6246-#6250)

## Reconciliation Matrix

| Audited Priority | Original Issue | Current Status | Follow-up Needed | Evidence |
|---|---:|---|---|---|
| quota/fairness tests | #6224 | Complete | No | direct policy tests present in runtime-guards |
| shared `kamn-types` `AgentDid` | #6226 | Complete | No | `kamn-types` crate exists and used |
| HKDF-SHA256 migration | #6227 | Complete | No | HKDF path and legacy decrypt compatibility tests present |
| CRLF request-builder hardening | #5929 | Complete | No | header/path validation + regression tests present |
| coverage thresholds | #6229 | Partial | Yes (`#6247`) | low minima remain for weak targets |
| PR E2E smoke coverage | #6230 | Partial | Yes (`#6248`) | PR path does not fully execute SDK/MCP lanes |
| kamn-core extraction wave 1 | #6231 | Partial | Yes (`#6249`) | compatibility shim surface remains in `kamn-core` |
| README reduction | #6232 | Complete | No | README under 200 lines |
| lane-script migration | #6233 | Partial | Yes (`#6250`) | shell/workflow/template ratio still above Rust |
| observability TLS default | #6234 | Complete | No | production fail-closed TLS defaults and tests present |

## Baseline Metrics (pre-follow-up)

- Shell/workflow/template LOC: `248615`
- Rust LOC (`crates/**/*.rs`): `237231`
- Shell-to-Rust ratio: `1.0480`
- Weak critical-path minima snapshot:
  - `crates/kamn-node/src/signer.rs`: line `4.5`, function `8.5`
  - `crates/kamn-node/src/runtime_orchestration.rs`: line `15.0`, function `18.0`
  - `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs`: line `27.0`, function `22.0`

## Follow-Up Work Items

- Story: #6246
- Task: #6247 (critical-path threshold ratchet)
- Task: #6248 (full PR E2E smoke coverage)
- Task: #6249 (wave-2 extraction and shim retirement)
- Task: #6250 (shell-surface ratio reduction + non-regression gate)

## Issue #6247 Threshold Ratchet (Implemented)

The critical-path gate remains deterministic and fail-closed while thresholds were raised across every target.

| Target | Previous Min (Line/Function) | New Min (Line/Function) | Measured Actual (Line/Function) |
|---|---|---|---|
| `crates/kamn-core/src/direct_message_crypto.rs` | `60.0 / 60.0` | `80.0 / 80.0` | `96.30 / 90.91` |
| `crates/kamn-core/src/group_channel_crypto.rs` | `50.0 / 50.0` | `55.0 / 55.0` | `58.63 / 56.60` |
| `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs` | `27.0 / 22.0` | `37.0 / 32.0` | `44.76 / 40.98` |
| `crates/kamn-node/src/runtime_orchestration.rs` | `15.0 / 18.0` | `20.0 / 28.0` | `24.46 / 28.95` |
| `crates/kamn-node/src/service_api_endpoint.rs` | `45.0 / 24.0` | `46.0 / 25.0` | `47.80 / 25.64` |
| `crates/kamn-node/src/signer.rs` | `4.5 / 8.5` | `15.0 / 19.0` | `37.12 / 48.72` |

Rationale:
- Weakest targets received at least one +10-point minimum increase (`http_transport`, `runtime_orchestration`, `signer`).
- Gate-selected tests were expanded to execute additional deterministic policy branches in `signer`, `runtime_orchestration`, and `http_transport`.
- No threshold was raised beyond measured coverage headroom.
