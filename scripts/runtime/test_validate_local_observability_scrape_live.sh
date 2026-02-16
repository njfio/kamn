#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_observability_scrape_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_observability_scrape_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local observability scrape live validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local observability scrape live policy checker script to be executable" >&2
  exit 1
fi

if ! grep -q 'local_observability_scrape_live_contract.py" run-lane' "$VALIDATION_SCRIPT"; then
  echo "expected local observability scrape validation wrapper to dispatch to python run-lane contract" >&2
  exit 1
fi

dry_run_report="$TMP_DIR/local-observability-scrape-live-summary.dry-run.json"
dry_run_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds 60 \
    --output-json "$dry_run_report"
)"
if ! printf '%s\n' "$dry_run_output" | grep -q '^status=pass$'; then
  echo "expected local observability scrape dry-run status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^final_decision=GO$'; then
  echo "expected local observability scrape dry-run final decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local observability scrape dry-run mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^lane_profile=standard$'; then
  echo "expected local observability scrape dry-run lane profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^readiness_probe_status=verified$'; then
  echo "expected local observability scrape dry-run readiness probe marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^readiness_failure_drill_status=verified$'; then
  echo "expected local observability scrape dry-run readiness failure-drill marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^readiness_reason_taxonomy_status=verified$'; then
  echo "expected local observability scrape dry-run readiness reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^degradation_taxonomy_status=verified$'; then
  echo "expected local observability scrape dry-run degradation taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^degradation_reason_codes_csv=none,readiness_transport_dependency_unhealthy,readiness_signer_dependency_unhealthy,readiness_commit_dependency_unhealthy,readiness_runtime_health_degraded$'; then
  echo "expected local observability scrape dry-run degradation reason-code taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^scrape_failure_taxonomy_status=verified$'; then
  echo "expected local observability scrape dry-run scrape-failure taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^scrape_failure_taxonomy_csv=readiness_failure_drill_status,stream_reconnect_churn_status,queue_bound_budget_status$'; then
  echo "expected local observability scrape dry-run scrape-failure taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^stream_reconnect_churn_status=verified$'; then
  echo "expected local observability scrape dry-run stream reconnect churn marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^queue_bound_budget_status=verified$'; then
  echo "expected local observability scrape dry-run queue-bound budget marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^observability_tls_route_contract_status=verified$'; then
  echo "expected local observability scrape dry-run observability TLS route marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^local_heavy_soak_lane_status=not_enabled$'; then
  echo "expected local observability scrape dry-run soak lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^soak_iterations_requested=1$'; then
  echo "expected local observability scrape dry-run soak iteration request marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^soak_iterations_executed=0$'; then
  echo "expected local observability scrape dry-run soak iteration execution marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^execution_reason_code=dry_run_no_commands_executed$'; then
  echo "expected local observability scrape dry-run execution reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$dry_run_output" | grep -q '^command_count=0$'; then
  echo "expected local observability scrape dry-run command count marker" >&2
  exit 1
fi

python3 - "$dry_run_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-observability-scrape-live-report.v1":
    raise SystemExit("unexpected local observability scrape dry-run schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local observability scrape dry-run status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local observability scrape dry-run final_decision=GO")
if payload.get("lane_mode") != "dry-run":
    raise SystemExit("expected local observability scrape dry-run lane_mode=dry-run")
if payload.get("lane_profile") != "standard":
    raise SystemExit("expected local observability scrape dry-run lane_profile=standard")
if payload.get("local_heavy_soak_lane_status") != "not_enabled":
    raise SystemExit("expected local observability scrape dry-run local_heavy_soak_lane_status=not_enabled")
if payload.get("soak_iterations_requested") != 1:
    raise SystemExit("expected local observability scrape dry-run soak_iterations_requested=1")
if payload.get("soak_iterations_executed") != 0:
    raise SystemExit("expected local observability scrape dry-run soak_iterations_executed=0")
if payload.get("stream_reconnect_churn_status") != "verified":
    raise SystemExit("expected local observability scrape dry-run stream_reconnect_churn_status=verified")
if payload.get("queue_bound_budget_status") != "verified":
    raise SystemExit("expected local observability scrape dry-run queue_bound_budget_status=verified")
if payload.get("observability_tls_route_contract_status") != "verified":
    raise SystemExit("expected local observability scrape dry-run observability_tls_route_contract_status=verified")
if payload.get("degradation_taxonomy_status") != "verified":
    raise SystemExit("expected local observability scrape dry-run degradation_taxonomy_status=verified")
if payload.get("degradation_reason_codes_csv") != "none,readiness_transport_dependency_unhealthy,readiness_signer_dependency_unhealthy,readiness_commit_dependency_unhealthy,readiness_runtime_health_degraded":
    raise SystemExit("expected local observability scrape dry-run degradation_reason_codes_csv taxonomy")
if payload.get("scrape_failure_taxonomy_status") != "verified":
    raise SystemExit("expected local observability scrape dry-run scrape_failure_taxonomy_status=verified")
if payload.get("scrape_failure_taxonomy_csv") != "readiness_failure_drill_status,stream_reconnect_churn_status,queue_bound_budget_status":
    raise SystemExit("expected local observability scrape dry-run scrape_failure_taxonomy_csv taxonomy")
if payload.get("execution_reason_code") != "dry_run_no_commands_executed":
    raise SystemExit("expected local observability scrape dry-run reason code")
if payload.get("command_count") != 0:
    raise SystemExit("expected local observability scrape dry-run command_count=0")
if payload.get("commands") != []:
    raise SystemExit("expected local observability scrape dry-run command list to be empty")
PY

policy_report="$TMP_DIR/local-observability-scrape-live-policy.dry-run.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$dry_run_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local observability scrape dry-run policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local observability scrape dry-run policy final decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_observability_scrape_policy_status=verified$'; then
  echo "expected local observability scrape dry-run policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes=none$'; then
  echo "expected local observability scrape dry-run policy reason code marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-observability-scrape-live-policy-report.v1":
    raise SystemExit("unexpected local observability scrape policy schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local observability scrape policy status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local observability scrape policy final_decision=GO")
if payload.get("local_observability_scrape_policy_status") != "verified":
    raise SystemExit("expected local_observability_scrape_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected local observability scrape policy reason code ['none']")
PY

soak_dry_run_report="$TMP_DIR/local-observability-scrape-live-summary.soak-dry-run.json"
soak_dry_run_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --lane-profile soak \
    --soak-iterations 2 \
    --max-seconds 60 \
    --output-json "$soak_dry_run_report"
)"
if ! printf '%s\n' "$soak_dry_run_output" | grep -q '^lane_profile=soak$'; then
  echo "expected local observability scrape soak dry-run lane profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$soak_dry_run_output" | grep -q '^local_heavy_soak_lane_status=verified$'; then
  echo "expected local observability scrape soak dry-run status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$soak_dry_run_output" | grep -q '^soak_iterations_requested=2$'; then
  echo "expected local observability scrape soak dry-run requested-iterations marker" >&2
  exit 1
fi
if ! printf '%s\n' "$soak_dry_run_output" | grep -q '^soak_iterations_executed=0$'; then
  echo "expected local observability scrape soak dry-run executed-iterations marker" >&2
  exit 1
fi
if ! printf '%s\n' "$soak_dry_run_output" | grep -q '^stream_reconnect_churn_status=verified$'; then
  echo "expected local observability scrape soak dry-run stream reconnect churn marker" >&2
  exit 1
fi
if ! printf '%s\n' "$soak_dry_run_output" | grep -q '^queue_bound_budget_status=verified$'; then
  echo "expected local observability scrape soak dry-run queue-bound budget marker" >&2
  exit 1
fi
if ! printf '%s\n' "$soak_dry_run_output" | grep -q '^observability_tls_route_contract_status=verified$'; then
  echo "expected local observability scrape soak dry-run observability TLS route marker" >&2
  exit 1
fi

python3 - "$soak_dry_run_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("lane_profile") != "soak":
    raise SystemExit("expected local observability scrape soak dry-run lane_profile=soak")
if payload.get("local_heavy_soak_lane_status") != "verified":
    raise SystemExit("expected local observability scrape soak dry-run local_heavy_soak_lane_status=verified")
if payload.get("soak_iterations_requested") != 2:
    raise SystemExit("expected local observability scrape soak dry-run soak_iterations_requested=2")
if payload.get("soak_iterations_executed") != 0:
    raise SystemExit("expected local observability scrape soak dry-run soak_iterations_executed=0")
if payload.get("observability_tls_route_contract_status") != "verified":
    raise SystemExit("expected local observability scrape soak dry-run observability_tls_route_contract_status=verified")
PY

soak_policy_report="$TMP_DIR/local-observability-scrape-live-policy.soak-dry-run.json"
soak_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$soak_dry_run_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$soak_policy_report"
)"
if ! printf '%s\n' "$soak_policy_output" | grep -q '^status=ok$'; then
  echo "expected local observability scrape soak dry-run policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$soak_policy_output" | grep -q '^local_observability_scrape_policy_status=verified$'; then
  echo "expected local observability scrape soak dry-run policy status marker" >&2
  exit 1
fi

set +e
missing_opt_in_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode run \
    --max-seconds 60 2>&1
)"
missing_opt_in_code=$?
set -e
if [ "$missing_opt_in_code" -eq 0 ]; then
  echo "expected local observability scrape run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_opt_in_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_LOCAL_OBSERVABILITY_SCRAPE_OPT_IN=1'; then
  echo "expected deterministic opt-in failure marker for local observability scrape run mode" >&2
  exit 1
fi

set +e
invalid_mode_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode invalid 2>&1
)"
invalid_mode_code=$?
set -e
if [ "$invalid_mode_code" -eq 0 ]; then
  echo "expected local observability scrape validation to reject invalid mode" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_mode_output" | grep -q -- '--mode must be one of: dry-run, run'; then
  echo "expected deterministic invalid-mode marker for local observability scrape validation" >&2
  exit 1
fi

set +e
invalid_lane_profile_output="$(
  bash "$VALIDATION_SCRIPT" \
    --lane-profile invalid 2>&1
)"
invalid_lane_profile_code=$?
set -e
if [ "$invalid_lane_profile_code" -eq 0 ]; then
  echo "expected local observability scrape validation to reject invalid lane-profile" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_lane_profile_output" | grep -q -- '--lane-profile must be one of: standard, soak'; then
  echo "expected deterministic invalid lane-profile marker for local observability scrape validation" >&2
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
  echo "expected local observability scrape validation to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'KAMN_LOCAL_OBSERVABILITY_SCRAPE_MAX_SECONDS must be an integer'; then
  echo "expected deterministic invalid max-seconds marker for local observability scrape validation" >&2
  exit 1
fi

echo "local observability scrape live validation tests passed."
