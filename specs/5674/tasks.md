# Tasks: #5674 Remove Remaining agent-lib Stubs via Service/SDK Route Expansion

- [ ] T1 (Conformance/Functional, RED): add failing `kamn-node` route tests for accept/complete/fund/release payload + scope/authz matrix updates.
- [ ] T2 (Implementation, GREEN): add service endpoint route support and scope-policy updates for the four routes.
- [ ] T3 (Conformance/Functional, RED): add failing `kamn-sdk` client tests for new methods and deterministic response decoding.
- [ ] T4 (Implementation, GREEN): implement `ServiceApiClient` methods and response models for accept/complete/fund/release.
- [ ] T5 (Conformance/Integration, RED): add failing `kamn-agent-lib` tests proving former stubs are now routed through SDK.
- [ ] T6 (Implementation, GREEN): replace `KamnAgentHandle` and `ServiceApiHttpClient` stubs with SDK-backed implementations.
- [ ] T7 (Regression): run targeted suites for `kamn-node`, `kamn-sdk`, and `kamn-agent-lib` plus fmt/clippy.
- [ ] T8 (Process): update issue process log and PR evidence matrix with AC mapping + Red/Green excerpts.
