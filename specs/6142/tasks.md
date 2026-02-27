# Tasks: Issue #6142

## Ordered Tasks
- [x] T1 (RED/Conformance): Added `crates/kamn-core/tests/k8s_manifest_baseline_contract.rs` and ran `cargo test -p kamn-core --test k8s_manifest_baseline_contract -- --nocapture` to capture failing pre-remediation assertions.
- [x] T2 (Implementation): Added `kamn-service-api` deployment plus `Service`/`Ingress` resources and `/healthz` readiness/liveness probes in `deploy/k8s/kamn-node.yaml`; updated deployment docs.
- [x] T3 (GREEN/Regression): Re-ran `cargo test -p kamn-core --test k8s_manifest_baseline_contract -- --nocapture` with all assertions passing.
- [x] T4 (Verification): Executed `cargo fmt --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, `bash scripts/deploy/test_deployment_assets.sh`, and `bash scripts/deploy/test_validate_deployment_assets_live.sh`.
- [ ] T5 (Closure): Open PR, add AC->test mapping and RED/GREEN evidence, then close issue with measured outputs.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T3, T4
- Integration: T4 (when cross-module behavior is affected)
- Regression: T1, T3, T4
- Conformance: T1, T4, T5
