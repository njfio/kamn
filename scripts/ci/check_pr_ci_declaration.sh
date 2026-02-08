#!/usr/bin/env bash
set -euo pipefail

# Enforce CI-impact declaration in PR body when CI-related files are changed.

if [ "${GITHUB_EVENT_NAME:-}" != "pull_request" ]; then
  echo "Non-PR event; CI declaration check skipped."
  exit 0
fi

if [ -z "${GITHUB_EVENT_PATH:-}" ] || [ ! -f "${GITHUB_EVENT_PATH}" ]; then
  echo "GITHUB_EVENT_PATH is unavailable; CI declaration check skipped."
  exit 0
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for check_pr_ci_declaration.sh" >&2
  exit 2
fi

pr_body="$(jq -r '.pull_request.body // ""' "$GITHUB_EVENT_PATH")"

base_ref="${GITHUB_BASE_REF:-main}"
if git rev-parse --verify "origin/${base_ref}" >/dev/null 2>&1; then
  base_commit="$(git merge-base HEAD "origin/${base_ref}")"
  mapfile -t changed_files < <(git diff --name-only "${base_commit}...HEAD" | sed '/^$/d')
elif git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
  mapfile -t changed_files < <(git diff --name-only HEAD~1...HEAD | sed '/^$/d')
else
  mapfile -t changed_files < <(git ls-files | sed '/^$/d')
fi

ci_sensitive=false
for file in "${changed_files[@]}"; do
  case "$file" in
    .github/workflows/*|scripts/ci/*|.ci/*)
      ci_sensitive=true
      break
      ;;
  esac
done

if [ "$ci_sensitive" != true ]; then
  echo "No CI-sensitive file changes; CI declaration check passed."
  exit 0
fi

checked_no='^\s*-\s*\[[xX]\]\s*No CI scope impact\.?\s*$'
checked_yes='^\s*-\s*\[[xX]\]\s*CI scope impact present\s*\(.*\)?\.?\s*$'

no_count="$(printf '%s\n' "$pr_body" | grep -Eci "$checked_no" || true)"
yes_count="$(printf '%s\n' "$pr_body" | grep -Eci "$checked_yes" || true)"

if [ "$no_count" -gt 0 ] && [ "$yes_count" -gt 0 ]; then
  echo "PR body marks both CI-impact options as checked; choose exactly one." >&2
  exit 1
fi

if [ "$yes_count" -eq 0 ]; then
  echo "CI-sensitive changes detected, but PR body does not check 'CI scope impact present'." >&2
  echo "Update .github/pull_request_template.md section in the PR description." >&2
  exit 1
fi

require_nonempty_field() {
  local field="$1"
  local value
  value="$(
    printf '%s\n' "$pr_body" | awk -v field="$field" '
      index($0, field) == 1 {
        value = substr($0, length(field) + 1)
        sub(/^[[:space:]]+/, "", value)
        print value
        exit
      }
    '
  )"
  if [ -z "${value//[[:space:]]/}" ]; then
    echo "PR CI impact declaration field is empty: ${field}" >&2
    return 1
  fi
  return 0
}

status=0
require_nonempty_field "Workflow(s) touched:" || status=1
require_nonempty_field "Expected runtime delta:" || status=1
require_nonempty_field "Expected runner-minute delta:" || status=1
require_nonempty_field "Rollback plan if CI cost/runtime regresses:" || status=1

if [ "$status" -ne 0 ]; then
  exit 1
fi

echo "PR CI impact declaration check passed."
