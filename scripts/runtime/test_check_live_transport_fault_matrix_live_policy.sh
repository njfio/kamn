#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_live_transport_fault_matrix_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_transport_fault_matrix_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED"' EXIT
EXPECTED_REASON_TAXONOMY_VERSION="kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1"
EXPECTED_REASON_CODES_CSV="ci_fast_gate_failed,live_transport_fault_matrix_policy_command_count_invalid,live_transport_fault_matrix_policy_command_count_mismatch,live_transport_fault_matrix_policy_elapsed_seconds_invalid,live_transport_fault_matrix_policy_execution_reason_code_mismatch,live_transport_fault_matrix_policy_final_decision_invalid,live_transport_fault_matrix_policy_final_decision_mismatch,live_transport_fault_matrix_policy_lane_mode_invalid,live_transport_fault_matrix_policy_marker_missing,live_transport_fault_matrix_policy_peer_adapter_multi_process_validation_local_heavy_status_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_budget_exhausted_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_peer_integrity_fail_closed_reason_code_mismatch,live_transport_fault_matrix_policy_reason_codes_classification_mismatch,live_transport_fault_matrix_policy_reason_codes_invalid,live_transport_fault_matrix_policy_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_runtime_transport_mode_mismatch,live_transport_fault_matrix_policy_schema_mismatch,live_transport_fault_matrix_policy_status_invalid"

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected live transport fault matrix validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected live transport fault matrix policy checker script to be executable" >&2
  exit 1
fi

bash "$VALIDATION_SCRIPT" --mode dry-run --ci-fast-gate PASS --output-json "$TMP_REPORT" >/dev/null

policy_output="$({
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
} 2>&1)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected live transport fault matrix policy status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected live transport fault matrix policy final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^live_transport_fault_matrix_policy_status=verified$'; then
  echo "expected live transport fault matrix policy verification marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION$"; then
  echo "expected deterministic live transport fault matrix policy reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^reason_codes_csv=$EXPECTED_REASON_CODES_CSV$"; then
  echo "expected deterministic live transport fault matrix policy reason codes taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes=none$'; then
  echo "expected deterministic live transport fault matrix policy reason_codes=none marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected deterministic live transport fault matrix policy reason_codes_value=none marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1$'; then
  echo "expected deterministic live transport fault matrix policy peer-adapter reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_integrity_fail_closed_reason_code=p2p_transport_unknown_sender_peer$'; then
  echo "expected deterministic live transport fault matrix policy peer-integrity fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout$'; then
  echo "expected deterministic live transport fault matrix policy retry-timeout reason projection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted$'; then
  echo "expected deterministic live transport fault matrix policy retry-budget-exhausted reason projection marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^peer_adapter_multi_process_validation_local_heavy_status=required$'; then
  echo "expected deterministic live transport fault matrix policy peer-adapter multi-process local-heavy marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-transport-fault-matrix-policy-report.v1":
    raise SystemExit("unexpected policy schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("live_transport_fault_matrix_policy_status") != "verified":
    raise SystemExit("expected policy status marker")
if payload.get("reason_taxonomy_version") != "kamn.runtime.live-transport-fault-matrix-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason taxonomy marker in policy report")
if payload.get("reason_codes_csv") != "ci_fast_gate_failed,live_transport_fault_matrix_policy_command_count_invalid,live_transport_fault_matrix_policy_command_count_mismatch,live_transport_fault_matrix_policy_elapsed_seconds_invalid,live_transport_fault_matrix_policy_execution_reason_code_mismatch,live_transport_fault_matrix_policy_final_decision_invalid,live_transport_fault_matrix_policy_final_decision_mismatch,live_transport_fault_matrix_policy_lane_mode_invalid,live_transport_fault_matrix_policy_marker_missing,live_transport_fault_matrix_policy_peer_adapter_multi_process_validation_local_heavy_status_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_budget_exhausted_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch,live_transport_fault_matrix_policy_peer_adapter_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_peer_integrity_fail_closed_reason_code_mismatch,live_transport_fault_matrix_policy_reason_codes_classification_mismatch,live_transport_fault_matrix_policy_reason_codes_invalid,live_transport_fault_matrix_policy_reason_taxonomy_version_mismatch,live_transport_fault_matrix_policy_runtime_transport_mode_mismatch,live_transport_fault_matrix_policy_schema_mismatch,live_transport_fault_matrix_policy_status_invalid":
    raise SystemExit("expected deterministic reason codes taxonomy marker in policy report")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none in policy report")
if payload.get("peer_adapter_reason_taxonomy_version") != "kamn.runtime.peer-adapter-reason-taxonomy.v1":
    raise SystemExit("expected deterministic peer_adapter_reason_taxonomy_version marker in policy report")
if payload.get("peer_integrity_fail_closed_reason_code") != "p2p_transport_unknown_sender_peer":
    raise SystemExit("expected deterministic peer_integrity_fail_closed_reason_code marker in policy report")
if payload.get("peer_adapter_reason_projection_timeout_code") != "p2p_live_reconnect_retry_dial_timeout":
    raise SystemExit("expected deterministic peer_adapter_reason_projection_timeout_code marker in policy report")
if payload.get("peer_adapter_reason_projection_budget_exhausted_code") != "p2p_live_reconnect_retry_budget_exhausted":
    raise SystemExit("expected deterministic peer_adapter_reason_projection_budget_exhausted_code marker in policy report")
if payload.get("peer_adapter_multi_process_validation_local_heavy_status") != "required":
    raise SystemExit("expected deterministic peer_adapter_multi_process_validation_local_heavy_status marker in policy report")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["partition_rejoin_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$({
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS
} 2>&1)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered live transport fault matrix report to fail policy" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status'; then
  echo "expected deterministic fail-closed marker for tampered live transport fault matrix report" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q "^reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION$"; then
  echo "expected deterministic reason taxonomy marker for tampered live transport fault matrix policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=live_transport_fault_matrix_policy_marker_missing:partition_rejoin_status$'; then
  echo "expected deterministic normalized reason_codes_value marker for tampered live transport fault matrix policy validation" >&2
  exit 1
fi

TMP_CLASSIFICATION_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_CLASSIFICATION_TAMPERED"' EXIT
cp "$TMP_REPORT" "$TMP_CLASSIFICATION_TAMPERED"
python3 - "$TMP_CLASSIFICATION_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = ["transport_unclassified_failure"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
classification_tampered_output="$({
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_CLASSIFICATION_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS
} 2>&1)"
classification_tampered_code=$?
set -e
if [ "$classification_tampered_code" -eq 0 ]; then
  echo "expected unstable reason code classification tamper to fail policy" >&2
  exit 1
fi
if ! printf '%s\n' "$classification_tampered_output" | grep -q 'live_transport_fault_matrix_policy_reason_codes_classification_mismatch'; then
  echo "expected deterministic unstable reason classification marker for live transport fault matrix policy" >&2
  exit 1
fi

TMP_TIMEOUT_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_CLASSIFICATION_TAMPERED" "$TMP_TIMEOUT_TAMPERED"' EXIT
cp "$TMP_REPORT" "$TMP_TIMEOUT_TAMPERED"
python3 - "$TMP_TIMEOUT_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["peer_adapter_reason_projection_timeout_code"] = "tampered_timeout_code"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
timeout_tampered_output="$({
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TIMEOUT_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS
} 2>&1)"
timeout_tampered_code=$?
set -e
if [ "$timeout_tampered_code" -eq 0 ]; then
  echo "expected timeout reason projection tamper to fail policy" >&2
  exit 1
fi
if ! printf '%s\n' "$timeout_tampered_output" | grep -q 'live_transport_fault_matrix_policy_peer_adapter_reason_projection_timeout_code_mismatch'; then
  echo "expected deterministic timeout reason projection mismatch marker for live transport fault matrix policy" >&2
  exit 1
fi

TMP_PEER_INTEGRITY_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_CLASSIFICATION_TAMPERED" "$TMP_TIMEOUT_TAMPERED" "$TMP_PEER_INTEGRITY_TAMPERED"' EXIT
cp "$TMP_REPORT" "$TMP_PEER_INTEGRITY_TAMPERED"
python3 - "$TMP_PEER_INTEGRITY_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["peer_integrity_fail_closed_reason_code"] = "tampered_peer_integrity_reason"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
peer_integrity_tampered_output="$({
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_PEER_INTEGRITY_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS
} 2>&1)"
peer_integrity_tampered_code=$?
set -e
if [ "$peer_integrity_tampered_code" -eq 0 ]; then
  echo "expected peer-integrity reason tamper to fail policy" >&2
  exit 1
fi
if ! printf '%s\n' "$peer_integrity_tampered_output" | grep -q 'live_transport_fault_matrix_policy_peer_integrity_fail_closed_reason_code_mismatch'; then
  echo "expected deterministic peer-integrity reason mismatch marker for live transport fault matrix policy" >&2
  exit 1
fi

set +e
ci_fast_gate_fail_output="$({
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate FAIL
} 2>&1)"
ci_fast_gate_fail_code=$?
set -e
if [ "$ci_fast_gate_fail_code" -eq 0 ]; then
  echo "expected live transport fault matrix policy checker to fail when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$ci_fast_gate_fail_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for live transport fault matrix policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$ci_fast_gate_fail_output" | grep -q '^reason_codes_value=ci_fast_gate_failed$'; then
  echo "expected deterministic normalized reason_codes_value marker for ci-fast-gate failure" >&2
  exit 1
fi

echo "live transport fault matrix policy tests passed."
