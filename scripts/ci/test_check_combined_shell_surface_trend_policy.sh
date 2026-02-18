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
python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["script_budget"]["status"] = "pass"
payload["current"]["shell_to_rust_ratio"] = 0.3
payload["deltas"]["script_count"] = 0
payload["deltas"]["shell_line_total"] = 0
payload["deltas"]["shell_to_rust_ratio"] = 0.0
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

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
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected combined shell-surface policy checker reason_codes_value=none marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.ci.combined-shell-surface-trend-policy-reason-taxonomy.v1$'; then
  echo "expected combined shell-surface policy checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_csv=combined_shell_surface_budget_status_fail,combined_shell_surface_decline_window_fail_exceeded,combined_shell_surface_decline_window_warn_exceeded,combined_shell_surface_delta_ratio_invalid,combined_shell_surface_delta_script_count_invalid,combined_shell_surface_delta_shell_line_total_invalid,combined_shell_surface_ratio_delta_fail_exceeded,combined_shell_surface_ratio_delta_warn_exceeded,combined_shell_surface_ratio_fail_ceiling_exceeded,combined_shell_surface_ratio_invalid,combined_shell_surface_ratio_warn_ceiling_exceeded,combined_shell_surface_report_schema_mismatch,combined_shell_surface_rust_line_total_invalid,combined_shell_surface_script_count_delta_fail_exceeded,combined_shell_surface_script_count_delta_warn_exceeded,combined_shell_surface_script_count_invalid,combined_shell_surface_shell_line_total_delta_fail_exceeded,combined_shell_surface_shell_line_total_delta_warn_exceeded,combined_shell_surface_shell_line_total_invalid,combined_shell_surface_threshold_date_invalid,combined_shell_surface_threshold_file_stale,combined_shell_surface_threshold_order_invalid,combined_shell_surface_threshold_schema_mismatch,combined_shell_surface_threshold_value_invalid,combined_shell_surface_today_override_invalid$'; then
  echo "expected combined shell-surface policy checker reason code taxonomy marker" >&2
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
if payload.get("reason_taxonomy_version") != "kamn.ci.combined-shell-surface-trend-policy-reason-taxonomy.v1":
    raise SystemExit("expected reason taxonomy version in passing combined shell-surface policy report")
if payload.get("reason_codes_csv") != "combined_shell_surface_budget_status_fail,combined_shell_surface_decline_window_fail_exceeded,combined_shell_surface_decline_window_warn_exceeded,combined_shell_surface_delta_ratio_invalid,combined_shell_surface_delta_script_count_invalid,combined_shell_surface_delta_shell_line_total_invalid,combined_shell_surface_ratio_delta_fail_exceeded,combined_shell_surface_ratio_delta_warn_exceeded,combined_shell_surface_ratio_fail_ceiling_exceeded,combined_shell_surface_ratio_invalid,combined_shell_surface_ratio_warn_ceiling_exceeded,combined_shell_surface_report_schema_mismatch,combined_shell_surface_rust_line_total_invalid,combined_shell_surface_script_count_delta_fail_exceeded,combined_shell_surface_script_count_delta_warn_exceeded,combined_shell_surface_script_count_invalid,combined_shell_surface_shell_line_total_delta_fail_exceeded,combined_shell_surface_shell_line_total_delta_warn_exceeded,combined_shell_surface_shell_line_total_invalid,combined_shell_surface_threshold_date_invalid,combined_shell_surface_threshold_file_stale,combined_shell_surface_threshold_order_invalid,combined_shell_surface_threshold_schema_mismatch,combined_shell_surface_threshold_value_invalid,combined_shell_surface_today_override_invalid":
    raise SystemExit("expected deterministic reason_codes_csv marker in passing combined shell-surface policy report")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none in passing combined shell-surface policy report")
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
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_taxonomy_version=kamn.ci.combined-shell-surface-trend-policy-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker for tampered shell-surface policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=.*combined_shell_surface_ratio_fail_ceiling_exceeded'; then
  echo "expected reason_codes_value marker to include ratio fail reason for tampered shell-surface policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=.*combined_shell_surface_shell_line_total_delta_fail_exceeded'; then
  echo "expected reason_codes_value marker to include shell-line fail reason for tampered shell-surface policy output" >&2
  exit 1
fi

tampered_thresholds="$TMP_DIR/combined-shell-surface-trend-thresholds.invalid-order.json"
cp "$THRESHOLDS" "$tampered_thresholds"
python3 - "$tampered_thresholds" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["warn_script_count_increase"] = 20
payload["fail_script_count_increase"] = 10
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
invalid_threshold_output="$(bash "$CHECKER" --report-file "$report_file" --threshold-file "$tampered_thresholds" --output-json "$TMP_DIR/invalid-threshold-policy.json" 2>&1)"
invalid_threshold_code=$?
set -e
if [[ "$invalid_threshold_code" -eq 0 ]]; then
  echo "expected invalid threshold ordering to fail combined shell-surface policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_threshold_output" | grep -q 'combined_shell_surface_threshold_order_invalid'; then
  echo "expected deterministic threshold-order reason code for invalid combined shell-surface threshold fixture" >&2
  exit 1
fi

script_count_fail_report="$TMP_DIR/combined-shell-surface-trend-report.script-count-fail.json"
cp "$report_file" "$script_count_fail_report"
python3 - "$script_count_fail_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["deltas"]["script_count"] = 16
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
script_count_fail_output="$(bash "$CHECKER" --report-file "$script_count_fail_report" --threshold-file "$THRESHOLDS" --output-json "$TMP_DIR/script-count-fail-policy.json" 2>&1)"
script_count_fail_code=$?
set -e
if [[ "$script_count_fail_code" -eq 0 ]]; then
  echo "expected script-count fail threshold breach to fail combined shell-surface policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$script_count_fail_output" | grep -q 'combined_shell_surface_script_count_delta_fail_exceeded'; then
  echo "expected deterministic script-count fail reason code for combined shell-surface policy output" >&2
  exit 1
fi

ratio_delta_fail_report="$TMP_DIR/combined-shell-surface-trend-report.ratio-delta-fail.json"
cp "$report_file" "$ratio_delta_fail_report"
python3 - "$ratio_delta_fail_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["deltas"]["shell_to_rust_ratio"] = 0.03
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
ratio_delta_fail_output="$(bash "$CHECKER" --report-file "$ratio_delta_fail_report" --threshold-file "$THRESHOLDS" --output-json "$TMP_DIR/ratio-delta-fail-policy.json" 2>&1)"
ratio_delta_fail_code=$?
set -e
if [[ "$ratio_delta_fail_code" -eq 0 ]]; then
  echo "expected ratio-delta fail threshold breach to fail combined shell-surface policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$ratio_delta_fail_output" | grep -q 'combined_shell_surface_ratio_delta_fail_exceeded'; then
  echo "expected deterministic ratio-delta fail reason code for combined shell-surface policy output" >&2
  exit 1
fi

warn_report="$TMP_DIR/combined-shell-surface-trend-report.warn.json"
cp "$report_file" "$warn_report"
python3 - "$warn_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["deltas"]["script_count"] = 6
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

warn_output="$(bash "$CHECKER" --report-file "$warn_report" --threshold-file "$THRESHOLDS" --output-json "$TMP_DIR/warn-policy.json")"
if ! printf '%s\n' "$warn_output" | grep -q '^status=ok$'; then
  echo "expected warn-only combined shell-surface policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^policy_decision=WARN$'; then
  echo "expected warn-only combined shell-surface policy decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q '^trend_status=warn$'; then
  echo "expected warn-only combined shell-surface trend status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$warn_output" | grep -q 'combined_shell_surface_script_count_delta_warn_exceeded'; then
  echo "expected deterministic script-count warn reason code in combined shell-surface policy output" >&2
  exit 1
fi

tampered_threshold_values="$TMP_DIR/combined-shell-surface-trend-thresholds.invalid-value.json"
cp "$THRESHOLDS" "$tampered_threshold_values"
python3 - "$tampered_threshold_values" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fail_shell_to_rust_ratio"] = "not-a-number"
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
invalid_threshold_value_output="$(bash "$CHECKER" --report-file "$report_file" --threshold-file "$tampered_threshold_values" --output-json "$TMP_DIR/invalid-threshold-value-policy.json" 2>&1)"
invalid_threshold_value_code=$?
set -e
if [[ "$invalid_threshold_value_code" -eq 0 ]]; then
  echo "expected invalid threshold value fixture to fail combined shell-surface policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_threshold_value_output" | grep -q 'combined_shell_surface_threshold_value_invalid'; then
  echo "expected deterministic threshold-value reason code for invalid combined shell-surface threshold fixture" >&2
  exit 1
fi

stale_thresholds="$TMP_DIR/combined-shell-surface-trend-thresholds.stale.json"
cp "$THRESHOLDS" "$stale_thresholds"
python3 - "$stale_thresholds" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["threshold_refreshed_on"] = "2025-01-01"
payload["threshold_max_age_days"] = 30
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
stale_threshold_output="$(bash "$CHECKER" --report-file "$report_file" --threshold-file "$stale_thresholds" --today 2026-02-18 --output-json "$TMP_DIR/stale-threshold-policy.json" 2>&1)"
stale_threshold_code=$?
set -e
if [[ "$stale_threshold_code" -eq 0 ]]; then
  echo "expected stale threshold metadata to fail combined shell-surface policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$stale_threshold_output" | grep -q 'combined_shell_surface_threshold_file_stale'; then
  echo "expected deterministic threshold-stale reason code for stale threshold metadata" >&2
  exit 1
fi

decline_thresholds="$TMP_DIR/combined-shell-surface-trend-thresholds.decline-window.json"
cp "$THRESHOLDS" "$decline_thresholds"
python3 - "$decline_thresholds" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["threshold_refreshed_on"] = "2026-01-01"
payload["threshold_max_age_days"] = 365
payload["warn_non_declining_window_days"] = 14
payload["fail_non_declining_window_days"] = 30
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

decline_warn_report="$TMP_DIR/combined-shell-surface-trend-report.decline-warn.json"
cp "$report_file" "$decline_warn_report"
python3 - "$decline_warn_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["deltas"]["script_count"] = 1
payload["deltas"]["shell_line_total"] = 1
payload["deltas"]["shell_to_rust_ratio"] = 0.001
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

decline_warn_output="$(bash "$CHECKER" --report-file "$decline_warn_report" --threshold-file "$decline_thresholds" --today 2026-01-20 --output-json "$TMP_DIR/decline-warn-policy.json")"
if ! printf '%s\n' "$decline_warn_output" | grep -q '^policy_decision=WARN$'; then
  echo "expected WARN decision when non-declining window exceeds warn threshold" >&2
  exit 1
fi
if ! printf '%s\n' "$decline_warn_output" | grep -q 'combined_shell_surface_decline_window_warn_exceeded'; then
  echo "expected deterministic decline-window warn reason code for non-declining trajectory" >&2
  exit 1
fi

set +e
decline_fail_output="$(bash "$CHECKER" --report-file "$decline_warn_report" --threshold-file "$decline_thresholds" --today 2026-02-20 --output-json "$TMP_DIR/decline-fail-policy.json" 2>&1)"
decline_fail_code=$?
set -e
if [[ "$decline_fail_code" -eq 0 ]]; then
  echo "expected NO-GO decision when non-declining window exceeds fail threshold" >&2
  exit 1
fi
if ! printf '%s\n' "$decline_fail_output" | grep -q 'combined_shell_surface_decline_window_fail_exceeded'; then
  echo "expected deterministic decline-window fail reason code for prolonged non-declining trajectory" >&2
  exit 1
fi

set +e
invalid_today_output="$(bash "$CHECKER" --report-file "$report_file" --threshold-file "$decline_thresholds" --today not-a-date --output-json "$TMP_DIR/invalid-today-policy.json" 2>&1)"
invalid_today_code=$?
set -e
if [[ "$invalid_today_code" -eq 0 ]]; then
  echo "expected invalid --today value to fail combined shell-surface policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_today_output" | grep -q 'combined_shell_surface_today_override_invalid'; then
  echo "expected deterministic today-override-invalid reason code for malformed --today input" >&2
  exit 1
fi

echo "combined shell-surface trend policy checker tests passed."
