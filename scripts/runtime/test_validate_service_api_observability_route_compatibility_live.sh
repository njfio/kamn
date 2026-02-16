#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_observability_route_compatibility_live.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api observability route compatibility validation script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/service-api-observability-route-compatibility-summary.json"
validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds 120 \
    --command-max-seconds 60 \
    --output-json "$report_file"
)"
for marker in \
  '^status=pass$' \
  '^final_decision=GO$' \
  '^lane_mode=dry-run$' \
  '^route_compatibility_matrix_status=verified$' \
  '^service_api_route_matrix_status=verified$' \
  '^observability_route_matrix_status=verified$' \
  '^route_parity_checkpoint_status=verified$' \
  '^fail_closed_checkpoint_status=verified$' \
  '^route_class_coverage_status=verified$' \
  '^fail_closed_status=verified$' \
  '^performance_budget_status=verified$' \
  '^execution_reason_code=dry_run_no_commands_executed$' \
  '^compatibility_row_count=11$'; do
  if ! printf '%s\n' "$validation_output" | grep -q "$marker"; then
    echo "expected service api observability route compatibility validation marker: $marker" >&2
    exit 1
  fi
done

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-observability-route-compatibility-live-report.v1":
    raise SystemExit("unexpected service api observability compatibility run-lane schema")
if payload.get("status") != "pass":
    raise SystemExit("expected run-lane status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected run-lane final_decision=GO")
if payload.get("matrix_schema_version") != "kamn.runtime.service-api-observability-route-compatibility-matrix.v1":
    raise SystemExit("expected matrix schema marker")
if payload.get("route_parity_checkpoint_status") != "verified":
    raise SystemExit("expected route_parity_checkpoint_status=verified")
if payload.get("fail_closed_checkpoint_status") != "verified":
    raise SystemExit("expected fail_closed_checkpoint_status=verified")
if payload.get("route_class_coverage_status") != "verified":
    raise SystemExit("expected route_class_coverage_status=verified")
if payload.get("compatibility_row_count") != 11:
    raise SystemExit("expected compatibility_row_count=11")
rows = payload.get("matrix_rows")
if not isinstance(rows, list) or len(rows) != 11:
    raise SystemExit("expected eleven matrix rows")
row_ids = {row.get("row_id") for row in rows}
expected_row_ids = {
    "api_healthz_get",
    "api_metrics_get",
    "api_websocket_upgrade_required_get",
    "api_messages_send_delete_method_not_allowed",
    "api_unknown_path_not_found",
    "observability_metrics_get",
    "observability_health_get",
    "observability_ready_get",
    "observability_stream_get",
    "observability_unknown_path_not_found",
    "observability_metrics_post_not_found",
}
if row_ids != expected_row_ids:
    raise SystemExit(f"unexpected matrix row set: {sorted(row_ids)}")
route_classes = {row.get("route_class") for row in rows}
expected_route_classes = {
    "service_api_health",
    "service_api_metrics",
    "service_api_websocket_upgrade",
    "service_api_method_guard",
    "service_api_route_not_found",
    "observability_metrics",
    "observability_health",
    "observability_readiness",
    "observability_stream",
    "observability_negative_path",
}
if route_classes != expected_route_classes:
    raise SystemExit(f"unexpected route class coverage set: {sorted(route_classes)}")
required_keys = {
    "row_id",
    "surface",
    "route_class",
    "route",
    "method",
    "expected_status",
    "expected_content_type",
    "evidence_test_selector",
    "compatibility_status",
}
missing = required_keys - set(rows[0].keys())
if missing:
    raise SystemExit(f"missing required matrix row keys: {sorted(missing)}")
if payload.get("command_count") != 0:
    raise SystemExit("expected dry-run command_count=0")
PY

set +e
invalid_mode_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode nope 2>&1
)"
invalid_mode_code=$?
set -e
if [ "$invalid_mode_code" -eq 0 ]; then
  echo "expected service api observability route compatibility validation to reject invalid mode" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_mode_output" | grep -q -- '--mode must be one of: dry-run, run'; then
  echo "expected deterministic invalid mode marker for service api observability route compatibility validation" >&2
  exit 1
fi

echo "service api observability route compatibility live validation tests passed."
