# Issue #5495 Evidence - Stale Closed-Issue Branch Trim

- Captured (UTC): 2026-02-21T15:21:07Z
- origin/main sha: f194571593524b1fc390e6c83e8e86ddb4c94a21
- pre_cleanup_remote_head_count: 51
- post_cleanup_remote_head_count: 50
- selected_candidate_issue: 1787
- selected_candidate_closed_at: 2026-02-11T21:17:05Z
- selected_candidate_branch: codex/issue-1787-remove-lifecycle-finality-delegate-wrappers
- selected_candidate_sha: 1df015f3c2e483ec4bcfb46ddda5ef2a9461906f
- selected_candidate_state: CLOSED
- selected_candidate_open_pr_head: no
- selected_candidate_merged_into_main: no

## Commands
```bash
git ls-remote --heads origin | wc -l
git ls-remote --heads origin 'refs/heads/codex/issue-*'
gh issue view <issue-id> --json state,closedAt
gh pr list --state open --json headRefName
git merge-base --is-ancestor <branch-sha> origin/main
gh api -X DELETE repos/njfio/kamn/git/refs/heads/<branch>
```

## Deterministic Candidate Filter Summary
- total_codex_issue_branches_seen: 48
- stale_safe_candidate_count: 48
- selection_rule: sort by issue_id asc, then closed_at asc, then branch name asc; choose first

## Selected Candidate Record
- 1787|2026-02-11T21:17:05Z|codex/issue-1787-remove-lifecycle-finality-delegate-wrappers|1df015f3c2e483ec4bcfb46ddda5ef2a9461906f|state=CLOSED|open_pr=no|merged_into_main=no
