#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_spec_phase6_evidence_policy.sh"
POLICY_DOC="$ROOT_DIR/docs/planning/spec-phase6-evidence-policy.md"
CONTRIBUTING_FILE="$ROOT_DIR/.github/CONTRIBUTING.md"
CI_TOOLS_FILE="$ROOT_DIR/scripts/ci/test_ci_tools.sh"

require_marker() {
  local file="$1"
  local marker="$2"
  local description="$3"
  if ! grep -Fq "$marker" "$file"; then
    echo "expected $description marker '$marker' in $file" >&2
    exit 1
  fi
}

if [ ! -x "$CHECKER" ]; then
  echo "expected spec phase6 evidence policy checker to be executable: $CHECKER" >&2
  exit 1
fi
if [ ! -f "$POLICY_DOC" ]; then
  echo "expected phase6 evidence policy documentation file: $POLICY_DOC" >&2
  exit 1
fi

required_policy_markers=(
  'spec_phase6_policy_version=kamn.spec-phase6-evidence-policy.v2'
  'spec_phase6_scope=specs/*.md closure-ready specs'
  'spec_phase6_canonical_section=## Phase 6 integration evidence'
  'spec_phase6_noncanonical_headings_fail_closed=true'
  'spec_phase6_required_execution_marker=Executed:'
  'spec_phase6_migration_plan_status=defined'
  'spec_phase6_policy_status=verified|fail-closed'
)

for marker in "${required_policy_markers[@]}"; do
  require_marker "$POLICY_DOC" "$marker" "phase6 evidence policy"
done

require_marker "$CONTRIBUTING_FILE" "scripts/ci/check_spec_phase6_evidence_policy.sh" "contributor closure command"
require_marker "$CONTRIBUTING_FILE" "Phase 6 integration evidence" "contributor closure guidance"

if ! grep -Fq 'bash "$ROOT_DIR/scripts/ci/test_check_spec_phase6_evidence_policy.sh"' "$CI_TOOLS_FILE"; then
  echo "expected CI tools regression lane to run phase6 evidence policy contract test" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

make_repo_fixture() {
  local target_root="$1"
  local status_line="$2"
  local include_phase6="$3"
  local include_executed="$4"

  mkdir -p "$target_root/specs"
  {
    echo "# Spec: Issue #9001 - Fixture"
    echo
    echo "$status_line"
    echo
    echo "## Acceptance criteria"
    echo "- [x] AC-1"
    if [ "$include_phase6" = "true" ]; then
      echo
      echo "## Phase 6 integration evidence"
      echo
      echo "- Wiring:"
      echo "  - Connected fixture entrypoint"
      if [ "$include_executed" = "true" ]; then
        echo "- Executed:"
        echo "  - \`cargo test -p kamn-core fixture_case\`"
      fi
    fi
  } > "$target_root/specs/9001-fixture.md"
}

PASS_ROOT="$TMP_DIR/pass-root"
make_repo_fixture "$PASS_ROOT" '- Status: Implemented' true true

PASS_REPORT="$TMP_DIR/pass-report.json"
pass_output="$(bash "$CHECKER" --repo-root "$PASS_ROOT" --output-json "$PASS_REPORT")"

if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected checker success status on compliant fixture" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^final_decision=GO$'; then
  echo "expected checker final_decision=GO on compliant fixture" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'; then
  echo "expected checker reason_codes=none on compliant fixture" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_taxonomy_version=kamn.ci.spec-phase6-evidence-policy-reason-taxonomy.v2$'; then
  echo "expected deterministic phase6 reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -Eq '^closure_ready_spec_count=[0-9]+$'; then
  echo "expected closure_ready_spec_count metric marker" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -Eq '^phase6_compliant_spec_count=[0-9]+$'; then
  echo "expected phase6_compliant_spec_count metric marker" >&2
  exit 1
fi

MISSING_PHASE6_ROOT="$TMP_DIR/missing-phase6-root"
make_repo_fixture "$MISSING_PHASE6_ROOT" '- Status: Implemented' false false

set +e
missing_phase6_output="$(bash "$CHECKER" --repo-root "$MISSING_PHASE6_ROOT" --output-json "$TMP_DIR/missing-phase6-report.json" 2>&1)"
missing_phase6_exit=$?
set -e

if [ "$missing_phase6_exit" -eq 0 ]; then
  echo "expected checker to fail when closure-ready spec omits Phase 6 section" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_phase6_output" | grep -q 'spec_phase6_missing_section'; then
  echo "expected deterministic spec_phase6_missing_section reason marker" >&2
  exit 1
fi

MISSING_EXECUTED_ROOT="$TMP_DIR/missing-executed-root"
make_repo_fixture "$MISSING_EXECUTED_ROOT" '- Status: Implemented' true false

set +e
missing_executed_output="$(bash "$CHECKER" --repo-root "$MISSING_EXECUTED_ROOT" --output-json "$TMP_DIR/missing-executed-report.json" 2>&1)"
missing_executed_exit=$?
set -e

if [ "$missing_executed_exit" -eq 0 ]; then
  echo "expected checker to fail when Phase 6 evidence omits executed markers" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_executed_output" | grep -q 'spec_phase6_missing_execution_markers'; then
  echo "expected deterministic spec_phase6_missing_execution_markers reason marker" >&2
  exit 1
fi

NONCANONICAL_HEADING_ROOT="$TMP_DIR/noncanonical-heading-root"
mkdir -p "$NONCANONICAL_HEADING_ROOT/specs"
cat > "$NONCANONICAL_HEADING_ROOT/specs/9002-fixture.md" <<'EOF'
# Spec: Issue #9002 - Fixture

- Status: Implemented

## Acceptance criteria
- [x] AC-1

## Phase 6 Evidence

- Wiring:
  - Connected fixture entrypoint
- Executed:
  - `cargo test -p kamn-core fixture_case`
EOF

set +e
noncanonical_heading_output="$(bash "$CHECKER" --repo-root "$NONCANONICAL_HEADING_ROOT" --output-json "$TMP_DIR/noncanonical-heading-report.json" 2>&1)"
noncanonical_heading_exit=$?
set -e

if [ "$noncanonical_heading_exit" -eq 0 ]; then
  echo "expected checker to fail when closure-ready spec uses a noncanonical Phase 6 heading" >&2
  exit 1
fi
if ! printf '%s\n' "$noncanonical_heading_output" | grep -q 'spec_phase6_noncanonical_section_heading'; then
  echo "expected deterministic spec_phase6_noncanonical_section_heading reason marker" >&2
  exit 1
fi

set +e
missing_output_json_output="$(bash "$CHECKER" --repo-root "$PASS_ROOT" 2>&1)"
missing_output_json_exit=$?
set -e

if [ "$missing_output_json_exit" -eq 0 ]; then
  echo "expected checker to fail when --output-json is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_output_json_output" | grep -q 'spec_phase6_output_json_required'; then
  echo "expected deterministic spec_phase6_output_json_required reason marker" >&2
  exit 1
fi

echo "spec phase6 evidence policy checker tests passed."
