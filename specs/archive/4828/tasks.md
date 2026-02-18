# Tasks — Issue #4828

- [x] T1 (Red): add/update failing tests before implementation.
  Evidence:
  - `python3 scripts/framework/test_declarative_policy_checker.py` failed with missing delegated marker assertion (`delegate_env=1` not captured).
  - `bash scripts/lib/test_exec_dispatch_registry.sh` failed with cohort mismatch (`expected 60 ... found 58`).
- [x] T2 (Green): implement compatibility delegation and cohort routing.
  Evidence:
  - `scripts/framework/declarative_policy_checker.py` supports legacy target delegation + bundle alias.
  - `scripts/lib/exec_dispatch.py` routes cohort-v1 eligible wrappers through declarative checker gateway.
- [x] T3 (Refactor): centralize compatibility migration behavior in shared gateway and shared registry contract test.
- [x] T4 (Verify): run deterministic suites and record outcomes.
  Evidence:
  - `python3 scripts/framework/test_declarative_policy_checker.py`
  - `bash scripts/lib/test_exec_dispatch_registry.sh`
  - `bash scripts/sdk/test_check_example_fixture_drift_policy.sh`
  - `bash scripts/bridge/test_generate_bridge_adapter_conformance_evidence_bundle.sh`
  - `bash scripts/runtime/test_check_service_api_prometheus_metrics_live_policy.sh`
  - `bash scripts/framework/test_contract_framework.sh`
  - `bash scripts/ci/test_ci_tools.sh`
