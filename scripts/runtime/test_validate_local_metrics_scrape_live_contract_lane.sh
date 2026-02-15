#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_local_metrics_scrape_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_metrics_scrape_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_metrics_scrape_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local metrics scrape contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local metrics scrape validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local metrics scrape policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/local-metrics-scrape-contract-lane-report.json"
policy_report="$TMP_DIR/local-metrics-scrape-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected local metrics scrape contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected local metrics scrape contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local metrics scrape contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^metrics_stream_readiness_status=verified$'; then
  echo "expected local metrics scrape contract lane metrics readiness marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^scrape_latency_budget_status=verified$'; then
  echo "expected local metrics scrape contract lane scrape latency budget marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_metrics_scrape_contract_status=verified$'; then
  echo "expected local metrics scrape contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_metrics_scrape_policy_status=verified$'; then
  echo "expected local metrics scrape contract lane policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^metrics_emission_reason_taxonomy_version=kamn.runtime.metrics-emission-reason-taxonomy.v1$'; then
  echo "expected local metrics scrape contract lane reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^metrics_emission_reason_codes_csv=metrics_stream_not_ready,metrics_scrape_latency_exceeded,metrics_payload_schema_mismatch$'; then
  echo "expected local metrics scrape contract lane reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=local_metrics_scrape_policy_marker_missing:local_scrape_probe_status$'; then
  echo "expected local metrics scrape contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.local-metrics-scrape-live-contract-lane-report.v1":
    raise SystemExit("unexpected local metrics scrape contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("metrics_stream_readiness_status") != "verified":
    raise SystemExit("expected metrics_stream_readiness_status=verified")
if lane_payload.get("scrape_latency_budget_status") != "verified":
    raise SystemExit("expected scrape_latency_budget_status=verified")
if lane_payload.get("local_metrics_scrape_contract_status") != "verified":
    raise SystemExit("expected local_metrics_scrape_contract_status=verified")
if lane_payload.get("local_metrics_scrape_policy_status") != "verified":
    raise SystemExit("expected local_metrics_scrape_policy_status=verified")
if lane_payload.get("metrics_emission_reason_taxonomy_version") != "kamn.runtime.metrics-emission-reason-taxonomy.v1":
    raise SystemExit("expected deterministic metrics_emission_reason_taxonomy_version marker")
if lane_payload.get("metrics_emission_reason_codes_csv") != "metrics_stream_not_ready,metrics_scrape_latency_exceeded,metrics_payload_schema_mismatch":
    raise SystemExit("expected deterministic metrics_emission_reason_codes_csv marker")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.local-metrics-scrape-live-policy-report.v1":
    raise SystemExit("unexpected local metrics scrape policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("local_metrics_scrape_policy_status") != "verified":
    raise SystemExit("expected local_metrics_scrape_policy_status=verified in policy report")
PY

if ! grep -q "check_local_metrics_scrape_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected local metrics scrape contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_local_metrics_scrape_live.sh" "$CONTRACT_LANE"; then
  echo "expected local metrics scrape contract lane to compose validation lane" >&2
  exit 1
fi

set +e
invalid_ci_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate MAYBE 2>&1
)"
invalid_ci_fast_gate_code=$?
set -e
if [ "$invalid_ci_fast_gate_code" -eq 0 ]; then
  echo "expected local metrics scrape contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for local metrics scrape contract lane" >&2
  exit 1
fi

set +e
blocked_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate FAIL 2>&1
)"
blocked_fast_gate_code=$?
set -e
if [ "$blocked_fast_gate_code" -eq 0 ]; then
  echo "expected local metrics scrape contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for local metrics scrape contract lane" >&2
  exit 1
fi

echo "local metrics scrape contract lane tests passed."
