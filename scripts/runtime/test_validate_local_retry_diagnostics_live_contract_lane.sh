#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_local_retry_diagnostics_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_retry_diagnostics_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_retry_diagnostics_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected local retry/diagnostics contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local retry/diagnostics validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local retry/diagnostics policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/runtime-local-retry-diagnostics-contract-lane-report.json"
policy_report="$TMP_DIR/runtime-local-retry-diagnostics-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 120 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected local retry/diagnostics contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected local retry/diagnostics contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected local retry/diagnostics contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_retry_diagnostics_policy_status=verified$'; then
  echo "expected local retry/diagnostics contract lane policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^local_retry_diagnostics_contract_status=verified$'; then
  echo "expected local retry/diagnostics contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^retry_readiness_status=verified$'; then
  echo "expected local retry/diagnostics contract lane retry readiness marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^retry_jitter_parity_status=verified$'; then
  echo "expected local retry/diagnostics contract lane retry jitter parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reason_taxonomy_version=kamn.runtime.local-retry-diagnostics-reason-taxonomy.v1$'; then
  echo "expected local retry/diagnostics contract lane reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^reason_codes_csv=local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,ci_local_network_budget_boundary_exceeded$'; then
  echo "expected local retry/diagnostics contract lane reason codes taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=local_retry_diagnostics_policy_marker_missing:correlation_diagnostics_status$'; then
  echo "expected local retry/diagnostics contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.local-retry-diagnostics-live-contract-lane-report.v1":
    raise SystemExit("unexpected local retry/diagnostics contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected contract lane final_decision=GO")
if lane_payload.get("local_retry_diagnostics_policy_status") != "verified":
    raise SystemExit("expected local_retry_diagnostics_policy_status=verified")
if lane_payload.get("local_retry_diagnostics_contract_status") != "verified":
    raise SystemExit("expected local_retry_diagnostics_contract_status=verified")
if lane_payload.get("retry_readiness_status") != "verified":
    raise SystemExit("expected retry_readiness_status=verified")
if lane_payload.get("retry_jitter_parity_status") != "verified":
    raise SystemExit("expected retry_jitter_parity_status=verified")
if lane_payload.get("reason_taxonomy_version") != "kamn.runtime.local-retry-diagnostics-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason_taxonomy_version marker")
if lane_payload.get("reason_codes_csv") != "local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,ci_local_network_budget_boundary_exceeded":
    raise SystemExit("expected deterministic reason_codes_csv marker")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.local-retry-diagnostics-live-policy-report.v1":
    raise SystemExit("unexpected local retry/diagnostics policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if policy_payload.get("local_retry_diagnostics_policy_status") != "verified":
    raise SystemExit("expected local_retry_diagnostics_policy_status=verified in policy report")
if policy_payload.get("reason_taxonomy_version") != "kamn.runtime.local-retry-diagnostics-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason_taxonomy_version marker in policy report")
if policy_payload.get("reason_codes_csv") != "local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,ci_local_network_budget_boundary_exceeded":
    raise SystemExit("expected deterministic reason_codes_csv marker in policy report")
PY

if ! grep -q "check_local_retry_diagnostics_live_policy.sh" "$CONTRACT_LANE"; then
  echo "expected local retry/diagnostics contract lane to compose policy checker" >&2
  exit 1
fi
if ! grep -q "validate_local_retry_diagnostics_live.sh" "$CONTRACT_LANE"; then
  echo "expected local retry/diagnostics contract lane to compose validation lane" >&2
  exit 1
fi

set +e
invalid_ci_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --ci-fast-gate MAYBE 2>&1
)"
invalid_ci_fast_gate_code=$?
set -e
if [ "$invalid_ci_fast_gate_code" -eq 0 ]; then
  echo "expected local retry/diagnostics contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for local retry/diagnostics contract lane" >&2
  exit 1
fi

set +e
budget_boundary_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --max-seconds 241 2>&1
)"
budget_boundary_code=$?
set -e
if [ "$budget_boundary_code" -eq 0 ]; then
  echo "expected local retry/diagnostics contract lane to reject max-seconds beyond ci-local boundary" >&2
  exit 1
fi
if ! printf '%s\n' "$budget_boundary_output" | grep -q 'max-seconds must be <= 240 for ci-local contract lane'; then
  echo "expected deterministic ci-local network budget boundary marker for local retry/diagnostics contract lane" >&2
  exit 1
fi

echo "local retry/diagnostics contract lane tests passed."
