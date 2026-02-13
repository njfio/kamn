#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_runtime_commit_live_lane.sh"
TMP_REPORT="$(mktemp)"
TMP_OUTPUT="$(mktemp)"
TMP_FINALITY_OUTPUT="$(mktemp)"
TMP_TIMEOUT_REPORT="$(mktemp)"
TMP_TIMEOUT_CLASS_DRIFT_REPORT="$(mktemp)"
TMP_TIMEOUT_ATTEMPT_DRIFT_REPORT="$(mktemp)"
TMP_TIMEOUT_FINALITY_FLAG_DRIFT_REPORT="$(mktemp)"
TMP_PROVIDER_DRIFT_REPORT="$(mktemp)"
TMP_SIGNER_ADAPTER_DRIFT_REPORT="$(mktemp)"
TMP_POLICY_REPORT="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_OUTPUT" "$TMP_FINALITY_OUTPUT" "$TMP_TIMEOUT_REPORT" "$TMP_TIMEOUT_CLASS_DRIFT_REPORT" "$TMP_TIMEOUT_ATTEMPT_DRIFT_REPORT" "$TMP_TIMEOUT_FINALITY_FLAG_DRIFT_REPORT" "$TMP_PROVIDER_DRIFT_REPORT" "$TMP_SIGNER_ADAPTER_DRIFT_REPORT" "$TMP_POLICY_REPORT" "$TMP_ERR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$CHECKER" ]; then
  echo "expected local runtime-commit live evidence policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$RUNNER" ]; then
  echo "expected local runtime-commit live lane runner to be executable" >&2
  exit 1
fi

KAMN_KOLME_LOCAL_HEAVY=1 \
  bash "$RUNNER" \
    --mode run \
    --skip-preflight \
    --live-command "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 printf 'status=submitted\nintegration_kolme_fork_live_node_submit_reaches_endpoint\n{\"pubkey\":\"proof\",\"nonce\":1,\"messages\":[]}\n'" \
    --finality-command "printf 'finality=final\n'" \
    --finality-retry-max-attempts 2 \
    --finality-retry-backoff-seconds 0 \
    --finality-max-seconds 3 \
    --max-seconds 5 \
    --output-json "$TMP_REPORT" \
    --live-output-file "$TMP_OUTPUT" \
    --finality-output-file "$TMP_FINALITY_OUTPUT" >/dev/null

python3 - "$TMP_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if summary.get("provider_contract_enforcement_mode") != "live-provider-only-v1":
    raise SystemExit("expected provider_contract_enforcement_mode=live-provider-only-v1 in live runtime summary")
if summary.get("provider_live_contract_marker") != "provider_client_contract=KolmeRuntimeCommitLiveProvider":
    raise SystemExit("expected deterministic provider_live_contract_marker in live runtime summary")
if summary.get("provider_live_contract_marker_present") is not True:
    raise SystemExit("expected provider_live_contract_marker_present=true in live runtime summary")
if summary.get("provider_in_memory_reference_detected") is not False:
    raise SystemExit("expected provider_in_memory_reference_detected=false in live runtime summary")
if summary.get("provider_signer_adapter_contract") != "KolmeForkSecp256k1SignerAdapter":
    raise SystemExit("expected provider_signer_adapter_contract=KolmeForkSecp256k1SignerAdapter in live runtime summary")
if summary.get("provider_signing_curve_contract") != "secp256k1":
    raise SystemExit("expected provider_signing_curve_contract=secp256k1 in live runtime summary")
if summary.get("provider_signing_profile_contract_version") != "v1":
    raise SystemExit("expected provider_signing_profile_contract_version=v1 in live runtime summary")
PY

python3 - "$TMP_REPORT" "$TMP_TIMEOUT_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
source["status"] = "fail"
source["reason_code"] = "live_finality_retry_exhausted_timeout"
source["finality_enabled"] = True
source["finality_evidence_marker_present"] = False
source["finality_retry_max_attempts"] = 2
source["finality_retry_backoff_seconds"] = 0
source["finality_retry_attempts_used"] = 2
source["finality_retry_exhausted"] = True
source["finality_retry_failure_class"] = "timeout"
pathlib.Path(sys.argv[2]).write_text(json.dumps(source, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

python3 - "$TMP_REPORT" "$TMP_PROVIDER_DRIFT_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
source["provider_in_memory_reference_detected"] = True
pathlib.Path(sys.argv[2]).write_text(json.dumps(source, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

python3 - "$TMP_REPORT" "$TMP_SIGNER_ADAPTER_DRIFT_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
source["provider_signer_adapter_contract"] = "SimulatedSignerAdapter"
pathlib.Path(sys.argv[2]).write_text(json.dumps(source, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

timeout_checker_output="$(
  python3 "$CHECKER" \
    --report-file "$TMP_TIMEOUT_REPORT" \
    --expected-final-decision NO-GO \
    --ci-fast-gate PASS \
    --require-reason-code live_finality_retry_exhausted_timeout \
    --output-json "$TMP_POLICY_REPORT"
)"
assert_eq "$(extract_value "$timeout_checker_output" "status")" "ok" "expected checker to accept deterministic timeout retry exhaustion mapping"
assert_eq "$(extract_value "$timeout_checker_output" "failed_checks")" "none" "expected timeout retry exhaustion mapping to have zero policy violations"

python3 - "$TMP_TIMEOUT_REPORT" "$TMP_TIMEOUT_CLASS_DRIFT_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
source["finality_retry_failure_class"] = "failed"
pathlib.Path(sys.argv[2]).write_text(json.dumps(source, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_TIMEOUT_CLASS_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-reason-code live_finality_retry_exhausted_timeout >"$TMP_ERR" 2>&1
timeout_class_drift_code=$?
set -e

if [ "$timeout_class_drift_code" -eq 0 ]; then
  echo "expected checker to fail closed when timeout retry failure class drifts" >&2
  exit 1
fi
if ! grep -q "finality_retry_failure_class_mismatch_for_timeout_reason" "$TMP_ERR"; then
  echo "expected timeout retry failure class drift reason from checker output" >&2
  exit 1
fi

python3 - "$TMP_TIMEOUT_REPORT" "$TMP_TIMEOUT_ATTEMPT_DRIFT_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
source["finality_retry_attempts_used"] = 1
pathlib.Path(sys.argv[2]).write_text(json.dumps(source, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_TIMEOUT_ATTEMPT_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-reason-code live_finality_retry_exhausted_timeout >"$TMP_ERR" 2>&1
timeout_attempt_drift_code=$?
set -e

if [ "$timeout_attempt_drift_code" -eq 0 ]; then
  echo "expected checker to fail closed when timeout retry attempts-used drifts" >&2
  exit 1
fi
if ! grep -q "finality_retry_attempts_used_mismatch_for_timeout_reason" "$TMP_ERR"; then
  echo "expected timeout retry attempts-used drift reason from checker output" >&2
  exit 1
fi

python3 - "$TMP_TIMEOUT_REPORT" "$TMP_TIMEOUT_FINALITY_FLAG_DRIFT_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

source = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
source["finality_enabled"] = False
pathlib.Path(sys.argv[2]).write_text(json.dumps(source, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
python3 "$CHECKER" \
  --report-file "$TMP_TIMEOUT_FINALITY_FLAG_DRIFT_REPORT" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --require-reason-code live_finality_retry_exhausted_timeout >"$TMP_ERR" 2>&1
timeout_finality_flag_drift_code=$?
set -e

if [ "$timeout_finality_flag_drift_code" -eq 0 ]; then
  echo "expected checker to fail closed when timeout retry reason is emitted without finality enabled" >&2
  exit 1
fi
if ! grep -q "finality_retry_timeout_reason_without_finality" "$TMP_ERR"; then
  echo "expected timeout retry reason-without-finality drift reason from checker output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_PROVIDER_DRIFT_REPORT" \
  --expected-final-decision GO \
  --ci-fast-gate PASS >"$TMP_ERR" 2>&1
provider_drift_code=$?
set -e

if [ "$provider_drift_code" -eq 0 ]; then
  echo "expected checker to fail closed for in-memory provider reference drift marker" >&2
  exit 1
fi
if ! grep -q "provider_in_memory_reference_detected" "$TMP_ERR"; then
  echo "expected provider_in_memory_reference_detected reason from checker output" >&2
  exit 1
fi

set +e
python3 "$CHECKER" \
  --report-file "$TMP_SIGNER_ADAPTER_DRIFT_REPORT" \
  --expected-final-decision GO \
  --ci-fast-gate PASS >"$TMP_ERR" 2>&1
signer_adapter_drift_code=$?
set -e

if [ "$signer_adapter_drift_code" -eq 0 ]; then
  echo "expected checker to fail closed for signer adapter contract drift marker" >&2
  exit 1
fi
if ! grep -q "provider_signer_adapter_contract_mismatch" "$TMP_ERR"; then
  echo "expected provider_signer_adapter_contract_mismatch reason from checker output" >&2
  exit 1
fi

# Regression: #2388
echo "local runtime-commit live evidence policy checker tests passed."
