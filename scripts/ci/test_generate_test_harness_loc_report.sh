#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/generate_test_harness_loc_report.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected test harness LOC report generator to be executable" >&2
  exit 1
fi

SCRIPTS_ROOT="$TMP_DIR/scripts"
mkdir -p "$SCRIPTS_ROOT/ci" "$SCRIPTS_ROOT/sdk"

cat >"$SCRIPTS_ROOT/ci/test_alpha.sh" <<'EOF_SCRIPT'
#!/usr/bin/env bash
echo "alpha"
EOF_SCRIPT

cat >"$SCRIPTS_ROOT/sdk/test_beta.sh" <<'EOF_SCRIPT'
#!/usr/bin/env bash
echo "beta"
EOF_SCRIPT

cat >"$SCRIPTS_ROOT/sdk/run_non_harness.sh" <<'EOF_SCRIPT'
#!/usr/bin/env bash
echo "ignore"
EOF_SCRIPT

REPORT_JSON="$TMP_DIR/test-harness-loc-report.json"
output="$(
  bash "$SCRIPT" \
    --scripts-root "$SCRIPTS_ROOT" \
    --output-json "$REPORT_JSON"
)"

if ! printf '%s\n' "$output" | grep -q '^status=ok$'; then
  echo "expected ok status from test harness LOC report generator" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q '^harness_script_count=2$'; then
  echo "expected deterministic harness_script_count=2" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q '^harness_shell_line_total=4$'; then
  echo "expected deterministic harness_shell_line_total=4" >&2
  exit 1
fi

if ! printf '%s\n' "$output" | grep -q "^report_file=$(realpath "$REPORT_JSON")$"; then
  echo "expected report_file marker from test harness LOC report generator" >&2
  exit 1
fi

python3 - "$REPORT_JSON" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.ci.test-harness-loc-report.v1":
    raise SystemExit("unexpected schema_version for test harness LOC report")
if report.get("harness_script_count") != 2:
    raise SystemExit("expected harness_script_count=2 in test harness LOC report")
if report.get("harness_shell_line_total") != 4:
    raise SystemExit("expected harness_shell_line_total=4 in test harness LOC report")
domains = report.get("domains")
if not isinstance(domains, dict):
    raise SystemExit("expected domains object in test harness LOC report")
if domains.get("ci", {}).get("script_count") != 1:
    raise SystemExit("expected ci domain script count marker")
if domains.get("sdk", {}).get("script_count") != 1:
    raise SystemExit("expected sdk domain script count marker")
paths = report.get("harness_scripts")
if paths != sorted(paths):
    raise SystemExit("expected deterministic sorted harness script list")
PY

echo "test harness LOC report generator tests passed."
