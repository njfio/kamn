#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_metrics_scrape_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local metrics scrape policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/local-metrics-scrape-live-summary.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_file" <<'JSON'
{
  "schema_version": "kamn.runtime.local-metrics-scrape-live-report.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_mode": "dry-run",
  "metrics_stream_readiness_status": "verified",
  "scrape_latency_budget_status": "verified",
  "scrape_latency_budget_seconds": 120,
  "max_observed_scrape_latency_seconds": 0,
  "metrics_emission_reason_taxonomy_version": "kamn.runtime.metrics-emission-reason-taxonomy.v1",
  "metrics_emission_reason_codes_csv": "metrics_stream_not_ready,metrics_scrape_latency_exceeded,metrics_payload_schema_mismatch",
  "local_scrape_probe_status": "verified",
  "prometheus_payload_status": "verified",
  "health_endpoint_status": "verified",
  "fail_closed_status": "verified",
  "ci_fast_gate_exclusion_status": "verified",
  "performance_budget_status": "verified",
  "execution_reason_code": "dry_run_no_commands_executed",
  "command_count": 0,
  "elapsed_seconds": 2
}
JSON

policy_report="$TMP_DIR/local-metrics-scrape-live-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local metrics scrape policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local metrics scrape policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_metrics_scrape_policy_status=verified$'; then
  echo "expected local metrics scrape policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-metrics-scrape-live-policy-report.v1":
    raise SystemExit("unexpected local metrics scrape policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("local_metrics_scrape_policy_status") != "verified":
    raise SystemExit("expected local_metrics_scrape_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
if payload.get("metrics_emission_reason_taxonomy_version") != "kamn.runtime.metrics-emission-reason-taxonomy.v1":
    raise SystemExit("expected deterministic metrics_emission_reason_taxonomy_version marker")
if payload.get("metrics_emission_reason_codes_csv") != "metrics_stream_not_ready,metrics_scrape_latency_exceeded,metrics_payload_schema_mismatch":
    raise SystemExit("expected deterministic metrics_emission_reason_codes_csv marker")
PY

tampered_report="$TMP_DIR/local-metrics-scrape-live-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["local_scrape_probe_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/local-metrics-scrape-live-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered local metrics scrape report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'local_metrics_scrape_policy_marker_missing:local_scrape_probe_status'; then
  echo "expected deterministic mismatch reason code for tampered local metrics scrape policy validation" >&2
  exit 1
fi

latency_budget_tampered_report="$TMP_DIR/local-metrics-scrape-live-summary.latency-budget.tampered.json"
cp "$report_file" "$latency_budget_tampered_report"
python3 - "$latency_budget_tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["scrape_latency_budget_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
latency_budget_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$latency_budget_tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/local-metrics-scrape-live-policy.latency-budget.tampered.json" 2>&1
)"
latency_budget_tampered_code=$?
set -e
if [ "$latency_budget_tampered_code" -eq 0 ]; then
  echo "expected scrape-latency budget tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$latency_budget_tampered_output" | grep -q 'local_metrics_scrape_policy_marker_missing:scrape_latency_budget_status'; then
  echo "expected deterministic scrape-latency budget mismatch reason code" >&2
  exit 1
fi

taxonomy_tampered_report="$TMP_DIR/local-metrics-scrape-live-summary.taxonomy.tampered.json"
cp "$report_file" "$taxonomy_tampered_report"
python3 - "$taxonomy_tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["metrics_emission_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
taxonomy_tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$taxonomy_tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/local-metrics-scrape-live-policy.taxonomy.tampered.json" 2>&1
)"
taxonomy_tampered_code=$?
set -e
if [ "$taxonomy_tampered_code" -eq 0 ]; then
  echo "expected metrics-emission taxonomy tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$taxonomy_tampered_output" | grep -q 'local_metrics_scrape_policy_metrics_emission_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic metrics-emission taxonomy mismatch reason code" >&2
  exit 1
fi

echo "local metrics scrape live policy checker tests passed."
