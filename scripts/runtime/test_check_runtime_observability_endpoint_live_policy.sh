#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_runtime_observability_endpoint_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_runtime_observability_endpoint_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected runtime observability endpoint live validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected runtime observability endpoint live policy checker script to be executable" >&2
  exit 1
fi

summary_report="$TMP_DIR/runtime-observability-endpoint-live-summary.json"
policy_report="$TMP_DIR/runtime-observability-endpoint-live-policy.json"
tampered_report="$TMP_DIR/runtime-observability-endpoint-live-summary.tampered.json"

bash "$VALIDATION_SCRIPT" --output-json "$summary_report" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected runtime observability endpoint live policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected runtime observability endpoint live policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runtime_observability_policy_status=verified$'; then
  echo "expected runtime observability endpoint live policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1$'; then
  echo "expected runtime observability endpoint live policy checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_csv=runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded$'; then
  echo "expected runtime observability endpoint live policy checker reason codes taxonomy marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.observability-endpoint-live-policy-report.v1":
    raise SystemExit("unexpected runtime observability endpoint policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("runtime_observability_policy_status") != "verified":
    raise SystemExit("expected runtime_observability_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
if payload.get("reason_taxonomy_version") != "kamn.runtime.observability-endpoint-reason-taxonomy.v1":
    raise SystemExit("expected deterministic runtime observability endpoint reason taxonomy marker")
if payload.get("reason_codes_csv") != "runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded":
    raise SystemExit("expected deterministic runtime observability endpoint reason codes taxonomy marker")
if payload.get("fail_closed_reason_codes_csv") != "observability_endpoint_not_found,observability_endpoint_malformed_request,observability_endpoint_idle_timeout":
    raise SystemExit("expected deterministic fail-closed reason-code taxonomy")
PY

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/runtime-observability-endpoint-live-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered runtime observability endpoint report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'runtime_observability_policy_final_decision_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered policy validation" >&2
  exit 1
fi

for marker_field in unknown_path_contract_status malformed_input_contract_status timeout_contract_status; do
  tampered_marker_report="$TMP_DIR/runtime-observability-endpoint-live-summary.${marker_field}.json"
  cp "$summary_report" "$tampered_marker_report"
  python3 - "$tampered_marker_report" "$marker_field" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
marker_field = sys.argv[2]
payload = json.loads(path.read_text(encoding="utf-8"))
payload[marker_field] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

  set +e
  tampered_marker_output="$(
    bash "$POLICY_CHECKER" \
      --report-file "$tampered_marker_report" \
      --expected-final-decision GO \
      --ci-fast-gate PASS \
      --output-json "$TMP_DIR/runtime-observability-endpoint-live-policy.${marker_field}.json" 2>&1
  )"
  tampered_marker_code=$?
  set -e
  if [ "$tampered_marker_code" -eq 0 ]; then
    echo "expected marker tamper for $marker_field to fail policy checker" >&2
    exit 1
  fi
  if ! printf '%s\n' "$tampered_marker_output" | grep -q "runtime_observability_policy_marker_missing:${marker_field}"; then
    echo "expected deterministic marker-missing reason code for tampered field $marker_field" >&2
    exit 1
  fi
done

tampered_taxonomy_report="$TMP_DIR/runtime-observability-endpoint-live-summary.fail-closed-taxonomy.json"
cp "$summary_report" "$tampered_taxonomy_report"
python3 - "$tampered_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fail_closed_reason_codes_csv"] = "observability_endpoint_not_found"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/runtime-observability-endpoint-live-policy.fail-closed-taxonomy.json" 2>&1
)"
tampered_taxonomy_code=$?
set -e
if [ "$tampered_taxonomy_code" -eq 0 ]; then
  echo "expected fail-closed taxonomy tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_taxonomy_output" | grep -q 'runtime_observability_policy_fail_closed_reason_codes_csv_mismatch'; then
  echo "expected deterministic fail-closed taxonomy mismatch reason code" >&2
  exit 1
fi

tampered_readiness_report="$TMP_DIR/runtime-observability-endpoint-live-summary.readiness.tampered.json"
cp "$summary_report" "$tampered_readiness_report"
python3 - "$tampered_readiness_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["endpoint_readiness_status"] = "stalled"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_readiness_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_readiness_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/runtime-observability-endpoint-live-policy.readiness.tampered.json" 2>&1
)"
tampered_readiness_code=$?
set -e
if [ "$tampered_readiness_code" -eq 0 ]; then
  echo "expected readiness drift tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_readiness_output" | grep -q 'runtime_observability_endpoint_readiness_progress_stalled'; then
  echo "expected deterministic runtime observability endpoint readiness drift reason code" >&2
  exit 1
fi

tampered_stream_parity_report="$TMP_DIR/runtime-observability-endpoint-live-summary.stream-parity.tampered.json"
cp "$summary_report" "$tampered_stream_parity_report"
python3 - "$tampered_stream_parity_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["stream_parity_status"] = "bypass-accepted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_stream_parity_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_stream_parity_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/runtime-observability-endpoint-live-policy.stream-parity.tampered.json" 2>&1
)"
tampered_stream_parity_code=$?
set -e
if [ "$tampered_stream_parity_code" -eq 0 ]; then
  echo "expected stream parity drift tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_stream_parity_output" | grep -q 'runtime_observability_stream_parity_bypass_detected'; then
  echo "expected deterministic runtime observability stream parity drift reason code" >&2
  exit 1
fi

tampered_budget_report="$TMP_DIR/runtime-observability-endpoint-live-summary.ci-budget.tampered.json"
cp "$summary_report" "$tampered_budget_report"
python3 - "$tampered_budget_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["max_seconds"] = 241
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_budget_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_budget_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/runtime-observability-endpoint-live-policy.ci-budget.tampered.json" 2>&1
)"
tampered_budget_code=$?
set -e
if [ "$tampered_budget_code" -eq 0 ]; then
  echo "expected ci-local budget boundary tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_budget_output" | grep -q 'ci_local_observability_endpoint_budget_boundary_exceeded'; then
  echo "expected deterministic ci-local budget boundary reason code" >&2
  exit 1
fi

echo "runtime observability endpoint live policy checker tests passed."
