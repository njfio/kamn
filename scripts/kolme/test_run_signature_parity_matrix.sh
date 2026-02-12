#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_signature_parity_matrix.py"
FIXTURE="$ROOT_DIR/fixtures/kolme_commit/signature_parity_vectors.json"
TMP_REPORT="$(mktemp)"
TMP_BAD_FIXTURE="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_BAD_FIXTURE"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected signature parity matrix runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE" ]; then
  echo "expected signature parity vector fixture to exist" >&2
  exit 1
fi

python3 "$RUNNER" --fixture "$FIXTURE" --output-json "$TMP_REPORT" >/dev/null

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.signature-parity-matrix-report.v1":
    raise SystemExit("unexpected signature parity matrix report schema")
if report.get("status") != "pass":
    raise SystemExit("expected signature parity matrix report status pass")
if report.get("vector_count", 0) < 3:
    raise SystemExit("expected at least three signature parity vectors")
cases = report.get("cases", [])
bad_cases = [
    case
    for case in cases
    if isinstance(case, dict)
    and case.get("vector_id") == "kolme_fork_primary_alpha_bad_signature"
]
if len(bad_cases) != 1:
    raise SystemExit("expected exactly one known-bad signature vector case")
bad_case = bad_cases[0]
if bad_case.get("observed_final_decision") != "NO-GO":
    raise SystemExit("expected known-bad signature vector decision NO-GO")
if "parity_signature_mismatch" not in bad_case.get("reason_codes", []):
    raise SystemExit("expected known-bad signature vector reason parity_signature_mismatch")
PY

python3 "$RUNNER" --fixture "$FIXTURE" --max-cases 1 --output-json "$TMP_REPORT" >/dev/null

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("vector_count") != 1:
    raise SystemExit("expected max-cases=1 to cap signature parity matrix vector_count")
if report.get("status") != "pass":
    raise SystemExit("expected capped signature parity matrix run to pass")
PY

cat >"$TMP_BAD_FIXTURE" <<'JSON'
{"schema_version":"bad","vectors":[]}
JSON

set +e
python3 "$RUNNER" --fixture "$TMP_BAD_FIXTURE" --output-json "$TMP_REPORT" >/dev/null 2>&1
bad_schema_exit_code=$?
set -e

if [ "$bad_schema_exit_code" -eq 0 ]; then
  echo "expected invalid signature parity fixture schema to fail closed" >&2
  exit 1
fi

echo "signature parity matrix runner tests passed."
