#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PR_TEMPLATE="$ROOT_DIR/.github/pull_request_template.md"
DECLARATION_CHECKER="$ROOT_DIR/scripts/ci/check_pr_ci_declaration.sh"

require_marker() {
  local file="$1"
  local marker="$2"
  local description="$3"
  if ! grep -Fq -- "$marker" "$file"; then
    echo "expected $description marker '$marker' in $file" >&2
    exit 1
  fi
}

if [[ ! -f "$PR_TEMPLATE" ]]; then
  echo "expected pull request template file at $PR_TEMPLATE" >&2
  exit 1
fi
if [[ ! -f "$DECLARATION_CHECKER" ]]; then
  echo "expected PR declaration checker script at $DECLARATION_CHECKER" >&2
  exit 1
fi

require_marker "$PR_TEMPLATE" "## Shell-Surface Impact Declaration" "PR template shell-surface declaration section"
require_marker "$PR_TEMPLATE" "- [ ] No shell-surface impact." "PR template shell-surface declaration checklist"
require_marker "$PR_TEMPLATE" "- [ ] Shell-surface impact present (explain below)." "PR template shell-surface declaration checklist"
require_marker "$PR_TEMPLATE" "shell_loc_delta_actual:" "PR template shell LOC delta marker"
require_marker "$PR_TEMPLATE" "rust_loc_delta_actual:" "PR template rust LOC delta marker"
require_marker "$PR_TEMPLATE" "shell_to_rust_ratio_delta_actual:" "PR template ratio delta marker"
require_marker "$PR_TEMPLATE" "shell_surface_ratio_target_status:" "PR template ratio target status marker"
require_marker "$PR_TEMPLATE" "shell_surface_mitigation_issue:" "PR template mitigation issue marker"

require_marker "$DECLARATION_CHECKER" "shell_loc_delta_actual:" "PR declaration checker shell LOC delta enforcement"
require_marker "$DECLARATION_CHECKER" "rust_loc_delta_actual:" "PR declaration checker rust LOC delta enforcement"
require_marker "$DECLARATION_CHECKER" "shell_to_rust_ratio_delta_actual:" "PR declaration checker ratio delta enforcement"
require_marker "$DECLARATION_CHECKER" "shell_surface_ratio_target_status:" "PR declaration checker ratio status enforcement"
require_marker "$DECLARATION_CHECKER" "shell_surface_mitigation_issue:" "PR declaration checker mitigation enforcement"

echo "PR template shell-surface marker contract tests passed."
