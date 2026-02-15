#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/ci/generate_combined_shell_surface_trend_report.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_combined_shell_surface_trend_policy.sh"
THRESHOLDS="$ROOT_DIR/fixtures/ci/combined_shell_surface_trend_thresholds.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ ! -x "$GENERATOR" ]]; then
  echo "expected combined shell-surface trend report generator to be executable" >&2
  exit 1
fi
if [[ ! -x "$CHECKER" ]]; then
  echo "expected combined shell-surface trend policy checker to be executable" >&2
  exit 1
fi
if [[ ! -f "$THRESHOLDS" ]]; then
  echo "expected combined shell-surface trend thresholds fixture to exist" >&2
  exit 1
fi

report_file="$TMP_DIR/combined-shell-surface-trend-report.json"
policy_file="$TMP_DIR/combined-shell-surface-trend-policy.json"

bash "$GENERATOR" --output-json "$report_file" >/dev/null

policy_output="$(bash "$CHECKER" --report-file "$report_file" --threshold-file "$THRESHOLDS" --output-json "$policy_file")"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected combined shell-surface policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^policy_decision=GO$'; then
  echo "expected combined shell-surface policy checker policy_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^trend_status=within$'; then
  echo "expected combined shell-surface policy checker trend_status=within marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes=none$'; then
  echo "expected combined shell-surface policy checker reason_codes=none marker" >&2
  exit 1
fi

python3 - "$policy_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.combined-shell-surface-trend-policy-report.v1":
    raise SystemExit("unexpected combined shell-surface policy schema")
if payload.get("status") != "ok":
    raise SystemExit("expected status=ok")
if payload.get("policy_decision") != "GO":
    raise SystemExit("expected policy_decision=GO")
if payload.get("trend_status") != "within":
    raise SystemExit("expected trend_status=within")
if payload.get("reason_codes") not in ([], None):
    raise SystemExit("expected empty reason_codes for passing policy")
PY

tampered_report="$TMP_DIR/combined-shell-surface-trend-report.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["current"]["shell_to_rust_ratio"] = 0.9
payload["deltas"]["shell_to_rust_ratio"] = 0.4
payload["deltas"]["shell_line_total"] = 100000
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" --threshold-file "$THRESHOLDS" --output-json "$TMP_DIR/tampered-policy.json" 2>&1)"
tampered_code=$?
set -e
if [[ "$tampered_code" -eq 0 ]]; then
  echo "expected tampered combined shell-surface report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'combined_shell_surface_ratio_fail_ceiling_exceeded'; then
  echo "expected deterministic ratio fail reason code for tampered report" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'combined_shell_surface_shell_line_total_delta_fail_exceeded'; then
  echo "expected deterministic shell_line_total fail reason code for tampered report" >&2
  exit 1
fi

echo "combined shell-surface trend policy checker tests passed."
