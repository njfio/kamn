# Issue #3789 Spec

- Title: Subtask: add fail-closed secure-mode config validation for observability routes
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Observability endpoint secure-mode startup must fail closed for invalid TLS configuration states, and runtime-network contract docs must stay synchronized with the deterministic reason taxonomy.

## Acceptance Criteria
- AC-1: Invalid observability endpoint secure-mode configurations fail closed with deterministic markers.
- AC-2: Valid TLS-required observability endpoint startup path serves required routes successfully.
- AC-3: Runtime-network docs declare secure-mode config markers and fail-closed taxonomy, pinned by docs-contract tests.
- AC-4: Unit, Functional, Integration, and Regression evidence is present and passing.

## Scope
In scope:
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs` as verification targets
- `crates/kamn-node/tests/observability_contracts_docs.rs`
- `docs/foundation/runtime-network.md` (contract surface; no new markers introduced in this issue)
- `specs/3789/{spec.md,plan.md,tasks.md}`

Out of scope:
- Certificate provisioning automation
- Deployment template rollout changes
- New dependencies

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | TLS secure-mode negative matrix tests (`missing cert`, `invalid key`, `invalid mode`, `plain HTTP to TLS listener`) | Each invalid state fails closed with deterministic error marker |
| C-02 | AC-2 | Integration | `integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes` | TLS-required endpoint serves `/metrics`, `/healthz`, `/readyz` over HTTPS with expected schema markers |
| C-03 | AC-3 | Unit/Integration | `observability_contracts_docs` runtime-network TLS contract tests | Runtime-network TLS env and taxonomy markers are present and aligned with `observability_endpoint.rs` |
| C-04 | AC-4 | Regression | `cargo fmt --check`, `cargo clippy -p kamn-node --all-targets -- -D warnings`, shell guardrails | Lint and shell-surface governance remain green |

## Test Mapping
- `cargo test -p kamn-node --test observability_contracts_docs`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_missing_cert_file -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_key_file -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_mode_value -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_rejects_plain_http_handshake -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3789.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3789.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3789.json`

## Success Metrics
- Secure-mode fail-closed taxonomy remains deterministic across TLS negative matrix states.
- Runtime-network TLS contract markers are fail-closed in docs-contract tests.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
