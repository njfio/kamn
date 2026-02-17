#!/usr/bin/env bash
set -euo pipefail

# Enforce CI-impact and shell-surface impact declarations in PR body when
# sensitive files are changed.

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

ci_force="${CI_DECLARATION_FORCE_SENSITIVE:-auto}"
shell_force="${SHELL_SURFACE_DECLARATION_FORCE_SENSITIVE:-auto}"

case "$ci_force" in
  true|false|auto) ;;
  *)
    echo "CI_DECLARATION_FORCE_SENSITIVE must be true|false|auto" >&2
    exit 2
    ;;
esac

case "$shell_force" in
  true|false|auto) ;;
  *)
    echo "SHELL_SURFACE_DECLARATION_FORCE_SENSITIVE must be true|false|auto" >&2
    exit 2
    ;;
esac

ci_sensitive=false
shell_surface_sensitive=false
ci_force_set=false
shell_force_set=false

if [ "$ci_force" = "true" ] || [ "$ci_force" = "false" ]; then
  ci_sensitive="$ci_force"
  ci_force_set=true
fi

if [ "$shell_force" = "true" ] || [ "$shell_force" = "false" ]; then
  shell_surface_sensitive="$shell_force"
  shell_force_set=true
fi

if [ "$ci_force_set" != true ] || [ "$shell_force_set" != true ]; then
  base_ref="${GITHUB_BASE_REF:-main}"
  if git rev-parse --verify "origin/${base_ref}" >/dev/null 2>&1; then
    base_commit="$(git merge-base HEAD "origin/${base_ref}")"
    mapfile -t changed_files < <(git diff --name-only "${base_commit}...HEAD" | sed '/^$/d')
  elif git rev-parse --verify HEAD~1 >/dev/null 2>&1; then
    mapfile -t changed_files < <(git diff --name-only HEAD~1...HEAD | sed '/^$/d')
  else
    mapfile -t changed_files < <(git ls-files | sed '/^$/d')
  fi

  for file in "${changed_files[@]}"; do
    if [ "$ci_force_set" != true ]; then
      case "$file" in
        .github/workflows/*|scripts/ci/*|.ci/*)
          ci_sensitive=true
          ;;
      esac
    fi

    if [ "$shell_force_set" != true ]; then
      case "$file" in
        scripts/*|.github/workflows/*|.github/ISSUE_TEMPLATE/*|.github/pull_request_template.md|AGENTS.md|.github/CONTRIBUTING.md)
          shell_surface_sensitive=true
          ;;
      esac
    fi

    if [ "$ci_force_set" != true ] && [ "$shell_force_set" != true ] && [ "$ci_sensitive" = true ] && [ "$shell_surface_sensitive" = true ]; then
      break
    fi
  done
fi

if [ "$ci_sensitive" != true ] && [ "$shell_surface_sensitive" != true ]; then
  echo "No CI-sensitive or shell-surface-sensitive file changes; declaration check passed."
  exit 0
fi

extract_field_value() {
  local field="$1"
  printf '%s\n' "$pr_body" | awk -v field="$field" '
    index($0, field) == 1 {
      value = substr($0, length(field) + 1)
      sub(/^[[:space:]]+/, "", value)
      print value
      exit
    }
  '
}

require_nonempty_field() {
  local field="$1"
  local value
  value="$(extract_field_value "$field")"
  if [ -z "${value//[[:space:]]/}" ]; then
    echo "PR declaration field is empty: ${field}" >&2
    return 1
  fi
  return 0
}

status=0

if [ "$ci_sensitive" = true ]; then
  checked_ci_no='^\s*-\s*\[[xX]\]\s*No CI scope impact\.?\s*$'
  checked_ci_yes='^\s*-\s*\[[xX]\]\s*CI scope impact present\s*\(.*\)?\.?\s*$'

  ci_no_count="$(printf '%s\n' "$pr_body" | grep -Eci "$checked_ci_no" || true)"
  ci_yes_count="$(printf '%s\n' "$pr_body" | grep -Eci "$checked_ci_yes" || true)"

  if [ "$ci_no_count" -gt 0 ] && [ "$ci_yes_count" -gt 0 ]; then
    echo "PR body marks both CI-impact options as checked; choose exactly one." >&2
    status=1
  elif [ "$ci_yes_count" -eq 0 ]; then
    echo "CI-sensitive changes detected, but PR body does not check 'CI scope impact present'." >&2
    echo "Update .github/pull_request_template.md section in the PR description." >&2
    status=1
  else
    require_nonempty_field "Workflow(s) touched:" || status=1
    require_nonempty_field "Expected runtime delta:" || status=1
    require_nonempty_field "Expected runner-minute delta:" || status=1
    require_nonempty_field "Rollback plan if CI cost/runtime regresses:" || status=1
  fi
fi

if [ "$shell_surface_sensitive" = true ]; then
  checked_shell_no='^\s*-\s*\[[xX]\]\s*No shell-surface impact\.?\s*$'
  checked_shell_yes='^\s*-\s*\[[xX]\]\s*Shell-surface impact present\s*\(.*\)?\.?\s*$'

  shell_no_count="$(printf '%s\n' "$pr_body" | grep -Eci "$checked_shell_no" || true)"
  shell_yes_count="$(printf '%s\n' "$pr_body" | grep -Eci "$checked_shell_yes" || true)"

  if [ "$shell_no_count" -gt 0 ] && [ "$shell_yes_count" -gt 0 ]; then
    echo "PR body marks both shell-surface impact options as checked; choose exactly one." >&2
    status=1
  elif [ "$shell_yes_count" -eq 0 ]; then
    echo "Shell-surface-sensitive changes detected, but PR body does not check 'Shell-surface impact present'." >&2
    echo "Update .github/pull_request_template.md shell-surface declaration section in the PR description." >&2
    status=1
  else
    require_nonempty_field "shell_loc_delta_actual:" || status=1
    require_nonempty_field "rust_loc_delta_actual:" || status=1
    require_nonempty_field "shell_to_rust_ratio_delta_actual:" || status=1

    if require_nonempty_field "shell_surface_ratio_target_status:"; then
      shell_ratio_target="$(extract_field_value "shell_surface_ratio_target_status:")"
      case "$shell_ratio_target" in
        improved|neutral|regressed_with_waiver) ;;
        *)
          echo "Invalid shell_surface_ratio_target_status: $shell_ratio_target (expected improved|neutral|regressed_with_waiver)" >&2
          status=1
          ;;
      esac
    else
      status=1
    fi

    require_nonempty_field "shell_surface_mitigation_issue:" || status=1
  fi
fi

if [ "$status" -ne 0 ]; then
  exit 1
fi

echo "PR declaration checks passed."
