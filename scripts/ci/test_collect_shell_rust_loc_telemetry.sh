#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COLLECTOR="$ROOT_DIR/scripts/ci/collect_shell_rust_loc_telemetry.sh"
GENERATOR="$ROOT_DIR/scripts/ci/generate_combined_shell_surface_trend_report.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ ! -x "$COLLECTOR" ]]; then
  echo "expected shell-rust LOC telemetry collector to be executable: $COLLECTOR" >&2
  exit 1
fi
if [[ ! -x "$GENERATOR" ]]; then
  echo "expected combined shell-surface trend generator to be executable: $GENERATOR" >&2
  exit 1
fi

report_file="$TMP_DIR/shell-rust-loc-telemetry.json"
output="$(bash "$COLLECTOR" --output-json "$report_file")"

if ! printf '%s\n' "$output" | grep -q '^status=ok$'; then
  echo "expected shell-rust LOC telemetry collector status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -q '^final_decision=GO$'; then
  echo "expected shell-rust LOC telemetry collector final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -q '^reason_taxonomy_version=kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1$'; then
  echo "expected shell-rust LOC telemetry collector reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -q '^reason_codes=none$'; then
  echo "expected shell-rust LOC telemetry collector reason_codes=none marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -Eq '^shell_line_total=[0-9]+$'; then
  echo "expected shell-rust LOC telemetry collector shell_line_total marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -Eq '^rust_line_total=[0-9]+$'; then
  echo "expected shell-rust LOC telemetry collector rust_line_total marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -Eq '^shell_to_rust_ratio=[0-9]+(\.[0-9]+)?$'; then
  echo "expected shell-rust LOC telemetry collector shell_to_rust_ratio marker" >&2
  exit 1
fi

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))

if payload.get("schema_version") != "kamn.ci.shell-rust-loc-telemetry-report.v1":
    raise SystemExit("unexpected shell-rust LOC telemetry report schema")
if payload.get("status") != "ok":
    raise SystemExit("expected status=ok")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("reason_taxonomy_version") != "kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason taxonomy marker")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none for passing telemetry report")
if payload.get("reason_codes") not in ([], None):
    raise SystemExit("expected empty reason_codes list for passing telemetry report")

metrics = payload.get("metrics", {})
if not isinstance(metrics.get("script_count"), int) or metrics["script_count"] <= 0:
    raise SystemExit("expected positive metrics.script_count")
if not isinstance(metrics.get("shell_line_total"), int) or metrics["shell_line_total"] <= 0:
    raise SystemExit("expected positive metrics.shell_line_total")
if not isinstance(metrics.get("rust_line_total"), int) or metrics["rust_line_total"] <= 0:
    raise SystemExit("expected positive metrics.rust_line_total")
if not isinstance(metrics.get("shell_to_rust_ratio"), float):
    raise SystemExit("expected float metrics.shell_to_rust_ratio")
if metrics.get("script_budget_status") != "pass":
    raise SystemExit("expected script_budget_status=pass for passing telemetry report")
PY

tampered_report="$TMP_DIR/combined-shell-surface-trend-report.tampered.json"
bash "$GENERATOR" --output-json "$tampered_report" >/dev/null
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["current"]["shell_to_rust_ratio"] = "invalid"
payload["script_budget"]["status"] = "fail"
payload["script_budget"]["checker_exit_code"] = 7
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$COLLECTOR" --report-file "$tampered_report" --output-json "$TMP_DIR/tampered-output.json" 2>&1)"
tampered_code=$?
set -e

if [[ "$tampered_code" -eq 0 ]]; then
  echo "expected shell-rust LOC telemetry collector to fail when report metrics/script-budget contract is invalid" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^status=fail$'; then
  echo "expected tampered shell-rust LOC telemetry collector output to include status=fail marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected tampered shell-rust LOC telemetry collector output to include final_decision=NO-GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'shell_rust_loc_telemetry_metric_type_invalid'; then
  echo "expected deterministic metric-type reason code in tampered telemetry collector output" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'shell_rust_loc_telemetry_script_budget_status_fail'; then
  echo "expected deterministic script-budget status reason code in tampered telemetry collector output" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'shell_rust_loc_telemetry_script_budget_exit_nonzero'; then
  echo "expected deterministic script-budget exit-code reason code in tampered telemetry collector output" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=.*shell_rust_loc_telemetry_metric_type_invalid'; then
  echo "expected reason_codes_value marker to include metric-type reason for tampered telemetry collector output" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=.*shell_rust_loc_telemetry_script_budget_status_fail'; then
  echo "expected reason_codes_value marker to include script-budget status reason for tampered telemetry collector output" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=.*shell_rust_loc_telemetry_script_budget_exit_nonzero'; then
  echo "expected reason_codes_value marker to include script-budget exit reason for tampered telemetry collector output" >&2
  exit 1
fi

fake_generator="$TMP_DIR/fake-generator.sh"
cat > "$fake_generator" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

# Intentionally skip writing output JSON to trigger deterministic missing-report failure.
echo "status=generated"
EOF
chmod +x "$fake_generator"

set +e
failure_output="$(bash "$COLLECTOR" --report-generator "$fake_generator" --output-json "$TMP_DIR/failure.json" 2>&1)"
failure_code=$?
set -e

if [[ "$failure_code" -eq 0 ]]; then
  echo "expected shell-rust LOC telemetry collector to fail when generator does not emit report" >&2
  exit 1
fi
if ! printf '%s\n' "$failure_output" | grep -q '^status=fail$'; then
  echo "expected failing telemetry collector output to include status=fail marker" >&2
  exit 1
fi
if ! printf '%s\n' "$failure_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected failing telemetry collector output to include final_decision=NO-GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$failure_output" | grep -q '^reason_taxonomy_version=kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1$'; then
  echo "expected failing telemetry collector output to include reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$failure_output" | grep -q 'shell_rust_loc_telemetry_report_missing'; then
  echo "expected deterministic report-missing reason code in failing telemetry collector output" >&2
  exit 1
fi
if ! printf '%s\n' "$failure_output" | grep -q '^reason_codes_value=.*shell_rust_loc_telemetry_report_missing'; then
  echo "expected reason_codes_value to include deterministic report-missing reason code" >&2
  exit 1
fi

echo "shell-rust LOC telemetry collector tests passed."
