# R27.46 R42 gap remediation and maintainability closure

## Milestone Summary
Execute closure of the R42 review findings with immediate stability fixes and tracked follow-through for larger structural concerns.

## Source Artifact
- Review source: `docs/review/gaps-and-issues-r42.md`

## Issue Hierarchy
- Epic:
  - `#5176` — Epic: R42 close stability and maintainability gaps with spec-driven execution
- Stories:
  - `#5177` — Story: remediate R42 immediate stability and hygiene findings
  - `#5178` — Story: execute R42 structural follow-through for module and governance maintainability
- Tasks:
  - `#5179` — Task: implement R42 immediate fixes (signer lock poisoning, ignored-test debt, PRD relocation, draft-spec review)
  - `#5180` — Task: decompose observability_endpoint.rs into focused modules with parity guarantees
  - `#5184` — Task: consolidate doc-contract tests into data-driven harness to reduce file sprawl
  - `#5183` — Task: automate merged-branch cleanup with safe retention policy
  - `#5181` — Task: audit public API and shell-vs-rust test surface with ratchet recommendations
  - `#5188` — Task: implement kamn-core public API surface report and growth ratchet
  - `#5189` — Task: migrate shell-first test wrappers to Rust-native suites and enforce shell-to-rust test ratio ratchet

## Governance Markers
- `shell_loc_hard_ceiling_env=.ci/shell-loc-hard-ceiling.env`
- `shell_rust_ratio_guardrail_env=.ci/shell-rust-ratio-guardrail.env`
- `shell_loc_hard_ceiling_max=130000`
- `warn_shell_rust_ratio_max=0.95`
- `fail_shell_rust_ratio_max=1.00`
