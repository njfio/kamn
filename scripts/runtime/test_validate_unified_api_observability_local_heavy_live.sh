#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_unified_api_observability_local_heavy_live.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected unified API-observability local-heavy validation script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/unified-api-observability-local-heavy-summary.json"
validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --ci-fast-gate PASS \
    --max-seconds 180 \
    --command-max-seconds 90 \
    --soak-iterations 2 \
    --output-json "$report_file"
)"
for marker in \
  '^status=pass$' \
  '^final_decision=GO$' \
  '^lane_mode=dry-run$' \
  '^ci_fast_gate_eligibility=eligible$' \
  '^compatibility_matrix_status=verified$' \
  '^compatibility_policy_status=verified$' \
  '^observability_soak_status=verified$' \
  '^observability_policy_status=verified$' \
  '^local_heavy_soak_lane_status=not_executed$' \
  '^soak_iterations_requested=2$' \
  '^soak_iterations_executed=0$' \
  '^local_heavy_runtime_budget_status=verified$' \
  '^run_mode_command_status=dry_run_no_commands_executed$' \
  '^run_mode_command_count=0$' \
  '^execution_reason_code=dry_run_no_commands_executed$'; do
  if ! printf '%s\n' "$validation_output" | grep -q "$marker"; then
    echo "expected unified API-observability local-heavy validation marker: $marker" >&2
    exit 1
  fi
done

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.unified-api-observability-local-heavy-live-report.v1":
    raise SystemExit("unexpected unified API-observability local-heavy run-lane schema")
if payload.get("status") != "pass":
    raise SystemExit("expected run-lane status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected run-lane final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected lane_mode=dry-run")
if payload.get("ci_fast_gate_eligibility") != "eligible":
    raise SystemExit("expected ci_fast_gate_eligibility=eligible")
if payload.get("local_heavy_soak_lane_status") != "not_executed":
    raise SystemExit("expected local_heavy_soak_lane_status=not_executed")
if payload.get("soak_iterations_requested") != 2:
    raise SystemExit("expected soak_iterations_requested=2")
if payload.get("soak_iterations_executed") != 0:
    raise SystemExit("expected soak_iterations_executed=0 in dry-run")
if payload.get("run_mode_command_count") != 0:
    raise SystemExit("expected run_mode_command_count=0 in dry-run")
if payload.get("execution_reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected deterministic dry-run reason code")
if payload.get("compatibility_report_schema_version") != "kamn.runtime.service-api-observability-route-compatibility-live-report.v1":
    raise SystemExit("expected compatibility report schema marker")
if payload.get("observability_report_schema_version") != "kamn.runtime.local-observability-scrape-live-report.v1":
    raise SystemExit("expected observability report schema marker")
if not isinstance(payload.get("artifact_paths"), dict):
    raise SystemExit("expected artifact_paths object")
PY

set +e
run_without_opt_in_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --ci-fast-gate PASS \
    --soak-iterations 1 2>&1
)"
run_without_opt_in_code=$?
set -e
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected unified API-observability local-heavy run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_UNIFIED_STACK_LOCAL_HEAVY_OPT_IN=1'; then
  echo "expected deterministic local-only opt-in marker for unified API-observability local-heavy run mode" >&2
  exit 1
fi

set +e
invalid_budget_output="$(
  bash "$VALIDATION_SCRIPT" \
    --max-seconds nope 2>&1
)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected unified API-observability local-heavy validation to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for unified API-observability local-heavy validation" >&2
  exit 1
fi

echo "unified API-observability local-heavy live validation tests passed."
