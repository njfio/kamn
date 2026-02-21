# Issue #5489 Evidence - Merged-Only Branch Hygiene

- Captured (UTC): 2026-02-21T14:51:16Z
- origin/main sha: 397d453104e55b3aef4bed0e187ff3c19d5a8025
- remote branch head count: 52
- merged non-main candidate count: 0

## Commands
```bash
git ls-remote --heads origin | wc -l
git rev-parse origin/main
git for-each-ref --format='%(refname:short)' refs/remotes/origin/codex/ | sed 's#^origin/##' | sort
for b in <codex-branches>; do sha=$(git rev-parse "origin/$b"); git merge-base --is-ancestor "$sha" origin/main && echo "$b"; done
```

## Merged Candidate Set
- None
