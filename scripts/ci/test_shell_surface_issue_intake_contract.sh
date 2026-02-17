#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
AGENTS_FILE="$ROOT_DIR/AGENTS.md"
CONTRIBUTING_FILE="$ROOT_DIR/.github/CONTRIBUTING.md"
ISSUE_TEMPLATES=(
  "$ROOT_DIR/.github/ISSUE_TEMPLATE/epic.md"
  "$ROOT_DIR/.github/ISSUE_TEMPLATE/story.md"
  "$ROOT_DIR/.github/ISSUE_TEMPLATE/task.md"
  "$ROOT_DIR/.github/ISSUE_TEMPLATE/subtask.md"
)

require_marker() {
  local file="$1"
  local marker="$2"
  local description="$3"
  if ! grep -Fq "$marker" "$file"; then
    echo "expected $description marker '$marker' in $file" >&2
    exit 1
  fi
}

if [[ ! -f "$AGENTS_FILE" ]]; then
  echo "expected AGENTS contract file at $AGENTS_FILE" >&2
  exit 1
fi
if [[ ! -f "$CONTRIBUTING_FILE" ]]; then
  echo "expected contributing contract file at $CONTRIBUTING_FILE" >&2
  exit 1
fi

require_marker "$AGENTS_FILE" "## Shell-Surface DoR Gate" "AGENTS shell-surface DoR gate"
require_marker "$AGENTS_FILE" "## Shell-Surface DoD Gate" "AGENTS shell-surface DoD gate"
require_marker "$AGENTS_FILE" "shell_loc_delta_estimate" "AGENTS shell-surface intake"
require_marker "$AGENTS_FILE" "rust_loc_delta_estimate" "AGENTS shell-surface intake"
require_marker "$AGENTS_FILE" "shell_to_rust_ratio_delta_estimate" "AGENTS shell-surface intake"
require_marker "$AGENTS_FILE" "shell_surface_mitigation_issue" "AGENTS shell-surface intake"
require_marker "$AGENTS_FILE" "shell_loc_delta_actual" "AGENTS shell-surface closure"
require_marker "$AGENTS_FILE" "rust_loc_delta_actual" "AGENTS shell-surface closure"
require_marker "$AGENTS_FILE" "shell_to_rust_ratio_delta_actual" "AGENTS shell-surface closure"
require_marker "$AGENTS_FILE" "shell_surface_ratio_target_status" "AGENTS shell-surface closure"

require_marker "$CONTRIBUTING_FILE" "## Shell-Surface DoR Gate" "CONTRIBUTING shell-surface DoR gate"
require_marker "$CONTRIBUTING_FILE" "## Shell-Surface DoD Gate" "CONTRIBUTING shell-surface DoD gate"
require_marker "$CONTRIBUTING_FILE" "shell_loc_delta_estimate" "CONTRIBUTING shell-surface intake"
require_marker "$CONTRIBUTING_FILE" "rust_loc_delta_estimate" "CONTRIBUTING shell-surface intake"
require_marker "$CONTRIBUTING_FILE" "shell_to_rust_ratio_delta_estimate" "CONTRIBUTING shell-surface intake"
require_marker "$CONTRIBUTING_FILE" "shell_surface_mitigation_issue" "CONTRIBUTING shell-surface intake"
require_marker "$CONTRIBUTING_FILE" "shell_loc_delta_actual" "CONTRIBUTING shell-surface closure"
require_marker "$CONTRIBUTING_FILE" "rust_loc_delta_actual" "CONTRIBUTING shell-surface closure"
require_marker "$CONTRIBUTING_FILE" "shell_to_rust_ratio_delta_actual" "CONTRIBUTING shell-surface closure"
require_marker "$CONTRIBUTING_FILE" "shell_surface_ratio_target_status" "CONTRIBUTING shell-surface closure"

for template in "${ISSUE_TEMPLATES[@]}"; do
  if [[ ! -f "$template" ]]; then
    echo "expected issue template file: $template" >&2
    exit 1
  fi
  require_marker "$template" "shell_loc_delta_estimate" "issue template shell-surface DoR"
  require_marker "$template" "rust_loc_delta_estimate" "issue template shell-surface DoR"
  require_marker "$template" "shell_to_rust_ratio_delta_estimate" "issue template shell-surface DoR"
  require_marker "$template" "shell_surface_mitigation_issue" "issue template shell-surface DoR"
done

echo "shell-surface issue intake contract tests passed."
