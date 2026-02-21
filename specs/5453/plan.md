# Issue #5453 Plan - Deterministic Empty-Milestone Closure

## Approach
1. Capture RED evidence of currently open milestones with `open_issues` counts.
2. Select only milestones where `open_issues=0`.
3. Close selected milestones via `gh api -X PATCH ... -f state=closed`.
4. Capture post-closure milestone state evidence.
5. Commit a planning artifact documenting selection criteria, pre/post tables, and commands used.
6. Run targeted docs regression suite.

## Affected Modules
- `docs/planning/2026-02-21-r27-empty-milestone-closure-wave.md` (new)
- `specs/5453/spec.md`
- `specs/5453/plan.md`
- `specs/5453/tasks.md`

## Risks / Mitigations
- Risk: accidentally closing a milestone that still has open issues.
  - Mitigation: strict `open_issues=0` filter and explicit pre-closure evidence capture.
- Risk: stale or ambiguous closure evidence.
  - Mitigation: include exact query commands and timestamped pre/post snapshots in artifact.

## Interfaces / Contracts
- GitHub milestone contract: closure only for zero-open milestones.
- Documentation contract: committed artifact includes deterministic evidence rows and command markers.

## Validation Strategy
- RED:
  - `gh api repos/njfio/kamn/milestones?state=open --paginate --jq '.[] | [.number,.title,.open_issues,.closed_issues,.state] | @tsv'`
- GREEN:
  - same command after closure wave; closed milestones removed from open list.
- REGRESSION:
  - `cargo test -p kamn-core --test docs_contract_wave3_harness -- --nocapture`
