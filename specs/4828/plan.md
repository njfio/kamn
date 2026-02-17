# Plan — Issue #4828

## Approach

1. Add compatibility delegation mode to declarative checker so legacy checker scripts can be executed transparently through a single gateway.
2. Add deterministic cohort-v1 eligibility routing in shared exec dispatcher for eligible wrappers.
3. Add/extend regression tests that enforce cohort size and validate delegation behavior in a sandbox.
4. Run targeted wrapper checks plus full `scripts/ci/test_ci_tools.sh` regression matrix.

## Affected Modules

- `scripts/lib/exec_dispatch.py`
- `scripts/lib/test_exec_dispatch_registry.sh`
- `scripts/framework/declarative_policy_checker.py`
- `scripts/framework/test_declarative_policy_checker.py`

## Risks / Mitigations

- Risk: wrapper behavior drift due to gateway routing.
  Mitigation: legacy delegate mode forwards stdout/stderr and exit status from legacy target unchanged.
- Risk: accidental cohort growth/shrink.
  Mitigation: enforce exact cohort size (`60`) in deterministic contract test.
- Risk: break existing policy checker mode in declarative checker.
  Mitigation: maintain policy-file path and add unit coverage for both policy and legacy modes.

## Interfaces / Contracts

- Declarative checker compatibility flags:
  - `--bundle-file` alias for report path compatibility
  - `--legacy-target`, `--legacy-interpreter`, repeated `--legacy-args-prefix`, passthrough args after `--`
- Delegation contract marker:
  - `KAMN_DECLARATIVE_POLICY_CHECKER_DELEGATE=1` exported to legacy target process.
- Cohort-v1 eligibility contract:
  - wrapper path contains `/check_` and ends `.sh`
  - python interpreter target ending `_contract.py` or `_policy_contract.py`
  - target line count `<= 500`

## ADR

No ADR required. This is implementation-scope routing and compatibility hardening without protocol/dependency changes.
