# R27 Program: operational hardening and live validation

## Milestone Summary
Program milestone for post-R26 production-operational closures focused on signal-safe daemon behavior, runtime modularization, signer reliability, and live-validation evidence with deterministic governance contracts.

## Issue Hierarchy
- Epic:
  - `#3812` — Epic: R27 operational hardening and live validation closure program
- Tasks:
  - `#5322` — Stabilize signer env guard tests by unifying test env lock across main_tests and signer module tests
  - `#5327` — Resume doc-contract harness consolidation to reduce include_str test-file count below 100
  - `#5336` — Close remaining R45 review gaps for signer flake, branch hygiene, and docs-contract wave 4

## Source Artifacts
- `docs/review/gaps-and-issues-r45.md`
- `docs/review/gaps-and-issues-r42.md`

## Governance Markers
- `signer_env_lock_domain=shared-kamn-node-test-lock`
- `docs_contract_include_str_file_budget=downward-only`
- `branch_hygiene_remote_head_budget=<100`
