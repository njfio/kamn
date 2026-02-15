#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_observability_scrape_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local observability scrape policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/local-observability-scrape-live-summary.json"
cat >"$report_file" <<'JSON'
{
  "schema_version": "kamn.runtime.local-observability-scrape-live-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "dry-run",
  "lane_profile": "standard",
  "scrape_probe_status": "verified",
  "metrics_content_type_status": "verified",
  "stream_lifecycle_status": "verified",
  "stream_reconnect_churn_status": "verified",
  "queue_bound_budget_status": "verified",
  "readiness_probe_status": "verified",
  "readiness_failure_drill_status": "verified",
  "readiness_reason_taxonomy_status": "verified",
  "local_heavy_soak_lane_status": "not_enabled",
  "soak_iterations_requested": 1,
  "soak_iterations_executed": 0,
  "fail_closed_status": "verified",
  "ci_fast_gate_exclusion_status": "verified",
  "performance_budget_status": "verified",
  "execution_reason_code": "dry_run_no_commands_executed",
  "command_count": 0,
  "elapsed_seconds": 2
}
JSON

policy_report="$TMP_DIR/local-observability-scrape-live-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local observability scrape policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local observability scrape policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_observability_scrape_policy_status=verified$'; then
  echo "expected local observability scrape policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-observability-scrape-live-policy-report.v1":
    raise SystemExit("unexpected local observability scrape policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("local_observability_scrape_policy_status") != "verified":
    raise SystemExit("expected local_observability_scrape_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
PY

tampered_report="$TMP_DIR/local-observability-scrape-live-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["readiness_failure_drill_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/local-observability-scrape-live-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered local observability scrape report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'local_observability_scrape_policy_marker_missing:readiness_failure_drill_status'; then
  echo "expected deterministic mismatch reason code for tampered local observability scrape policy validation" >&2
  exit 1
fi

tampered_queue_bound_report="$TMP_DIR/local-observability-scrape-live-summary.queue-bound.tampered.json"
cp "$report_file" "$tampered_queue_bound_report"
python3 - "$tampered_queue_bound_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["queue_bound_budget_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_queue_bound_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_queue_bound_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/local-observability-scrape-live-policy.queue-bound.tampered.json" 2>&1
)"
tampered_queue_bound_code=$?
set -e

if [ "$tampered_queue_bound_code" -eq 0 ]; then
  echo "expected tampered local observability queue-bound marker report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_queue_bound_output" | grep -q 'local_observability_scrape_policy_marker_missing:queue_bound_budget_status'; then
  echo "expected deterministic queue-bound marker mismatch reason code for tampered local observability policy validation" >&2
  exit 1
fi

tampered_soak_report="$TMP_DIR/local-observability-scrape-live-summary.soak.tampered.json"
cp "$report_file" "$tampered_soak_report"
python3 - "$tampered_soak_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["lane_mode"] = "run"
payload["lane_profile"] = "soak"
payload["local_heavy_soak_lane_status"] = "missing"
payload["soak_iterations_requested"] = 2
payload["soak_iterations_executed"] = 2
payload["execution_reason_code"] = "soak_run_mode_commands_executed"
payload["command_count"] = 10
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_soak_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_soak_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/local-observability-scrape-live-policy.soak.tampered.json" 2>&1
)"
tampered_soak_code=$?
set -e

if [ "$tampered_soak_code" -eq 0 ]; then
  echo "expected tampered local observability soak report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_soak_output" | grep -q 'local_observability_scrape_policy_marker_missing:local_heavy_soak_lane_status'; then
  echo "expected deterministic soak-marker mismatch reason code for tampered local observability policy validation" >&2
  exit 1
fi

echo "local observability scrape live policy checker tests passed."
