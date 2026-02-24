#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_e2e_live_workflow_contract.py"
WORKFLOW_FILE="$ROOT_DIR/.github/workflows/e2e-live.yml"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"

# Red-first contract: checker must exist and be executable.
test_harness_require_executable "$CHECKER" "expected e2e-live workflow contract checker to be executable"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

safe_log="$tmp_dir/safe.out"
if ! python3 "$CHECKER" \
  --workflow-file "$WORKFLOW_FILE" \
  --strategy-doc "$STRATEGY_DOC" >"$safe_log" 2>&1; then
  cat "$safe_log" >&2
  echo "expected repository baseline to satisfy e2e-live workflow contract checker" >&2
  exit 1
fi

grep -q '^status=pass$' "$safe_log"
grep -q '^final_decision=GO$' "$safe_log"
grep -q '^reason_codes_value=none$' "$safe_log"
grep -q '^e2e_live_workflow_contract_status=verified$' "$safe_log"

missing_toggle_workflow="$tmp_dir/missing-toggle.yml"
cp "$WORKFLOW_FILE" "$missing_toggle_workflow"
python3 - "$missing_toggle_workflow" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding='utf-8')
text = text.replace('KAMN_E2E_SDK_DIRECT_LIVE: "1"\n', '', 1)
path.write_text(text, encoding='utf-8')
PY

missing_toggle_log="$tmp_dir/missing-toggle.out"
if python3 "$CHECKER" \
  --workflow-file "$missing_toggle_workflow" \
  --strategy-doc "$STRATEGY_DOC" >"$missing_toggle_log" 2>&1; then
  cat "$missing_toggle_log" >&2
  echo "expected missing SDK-direct live toggle fixture to fail checker" >&2
  exit 1
fi
grep -q 'sdk_direct_live_toggle_missing' "$missing_toggle_log"

truncated_scenarios_workflow="$tmp_dir/truncated-scenarios.yml"
cp "$WORKFLOW_FILE" "$truncated_scenarios_workflow"
python3 - "$truncated_scenarios_workflow" <<'PY'
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding='utf-8')
text = re.sub(
    r'--scenarios S-01,S-02,S-03,S-04,S-05,S-06(?:,S-07,S-08,S-09,S-10,S-11,S-12,S-13,S-14,S-15)?',
    '--scenarios S-01,S-02,S-03,S-04,S-05,S-06',
    text,
    count=1,
)
path.write_text(text, encoding='utf-8')
PY

truncated_scenarios_log="$tmp_dir/truncated-scenarios.out"
if python3 "$CHECKER" \
  --workflow-file "$truncated_scenarios_workflow" \
  --strategy-doc "$STRATEGY_DOC" >"$truncated_scenarios_log" 2>&1; then
  cat "$truncated_scenarios_log" >&2
  echo "expected truncated SDK-direct scenario fixture to fail checker" >&2
  exit 1
fi
grep -q 'sdk_direct_scenarios_not_full_matrix' "$truncated_scenarios_log"

missing_external_exec_workflow="$tmp_dir/missing-external-exec.yml"
cp "$WORKFLOW_FILE" "$missing_external_exec_workflow"
python3 - "$missing_external_exec_workflow" <<'PY'
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding='utf-8')
text, count = re.subn(
    r'^[ \t]*--enable-external-execution[ \t]*\\?[ \t]*\n',
    '',
    text,
    count=1,
    flags=re.MULTILINE,
)
if count != 1:
    raise SystemExit("failed to remove --enable-external-execution fixture marker")
path.write_text(text, encoding='utf-8')
PY

missing_external_exec_log="$tmp_dir/missing-external-exec.out"
if python3 "$CHECKER" \
  --workflow-file "$missing_external_exec_workflow" \
  --strategy-doc "$STRATEGY_DOC" >"$missing_external_exec_log" 2>&1; then
  cat "$missing_external_exec_log" >&2
  echo "expected missing external-execution fixture to fail checker" >&2
  exit 1
fi
grep -q 'sdk_direct_external_execution_flag_missing' "$missing_external_exec_log"

missing_doc_marker="$tmp_dir/strategy-missing-marker.md"
cp "$STRATEGY_DOC" "$missing_doc_marker"
python3 - "$missing_doc_marker" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding='utf-8')
text = text.replace('## E2E Live Workflow Contract\n', '', 1)
path.write_text(text, encoding='utf-8')
PY

missing_doc_marker_log="$tmp_dir/strategy-missing-marker.out"
if python3 "$CHECKER" \
  --workflow-file "$WORKFLOW_FILE" \
  --strategy-doc "$missing_doc_marker" >"$missing_doc_marker_log" 2>&1; then
  cat "$missing_doc_marker_log" >&2
  echo "expected missing strategy marker fixture to fail checker" >&2
  exit 1
fi
grep -q 'ci_strategy_markers_missing' "$missing_doc_marker_log"

echo "e2e-live workflow contract checker tests passed."
