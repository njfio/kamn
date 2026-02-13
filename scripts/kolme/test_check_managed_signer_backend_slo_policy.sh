#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_managed_signer_backend_slo_policy.py"
GENERATOR="$ROOT_DIR/scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh"
TMP_DIR="$(mktemp -d)"
GO_BUNDLE="$TMP_DIR/go-bundle.json"
NO_GO_BUNDLE="$TMP_DIR/no-go-bundle.json"
GO_REPORT="$TMP_DIR/go-policy-report.json"
NO_GO_REPORT="$TMP_DIR/no-go-policy-report.json"
MALFORMED_BUNDLE="$TMP_DIR/malformed-bundle.json"
MALFORMED_REPORT="$TMP_DIR/malformed-policy-report.json"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { sub($1 "=",""); print; exit }'
}

if [ ! -x "$CHECKER" ]; then
  echo "expected managed-signer backend SLO policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$GENERATOR" ]; then
  echo "expected managed-signer backend SLO telemetry generator to be executable" >&2
  exit 1
fi

bash "$GENERATOR" \
  --output-file "$GO_BUNDLE" \
  --window-start-utc "2026-02-13T00:00:00Z" \
  --window-end-utc "2026-02-13T00:15:00Z" \
  --backend-name "kolme-managed-signer-primary" \
  --signer-profile "ops-primary" \
  --signer-key-source "managed-external" \
  --sample-count 100 \
  --timeout-events 0 \
  --unavailable-events 0 \
  --error-events 1 \
  --max-timeout-rate-bps 100 \
  --max-unavailable-rate-bps 100 \
  --max-error-rate-bps 200 \
  --ci-fast-gate PASS >/dev/null

go_output="$(
  python3 "$CHECKER" \
    --telemetry-bundle "$GO_BUNDLE" \
    --output-json "$GO_REPORT"
)"

if [ "$(extract_value "$go_output" "status")" != "ok" ]; then
  echo "expected GO policy status to be ok" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "final_decision")" != "GO" ]; then
  echo "expected GO policy final_decision=GO" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "reason_codes")" != "managed_signer_backend_slo_within_threshold" ]; then
  echo "expected deterministic GO policy reason code" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "remediation_markers")" != "managed_signer_backend_no_action_required" ]; then
  echo "expected deterministic GO remediation marker" >&2
  exit 1
fi

bash "$GENERATOR" \
  --output-file "$NO_GO_BUNDLE" \
  --window-start-utc "2026-02-13T00:15:00Z" \
  --window-end-utc "2026-02-13T00:30:00Z" \
  --backend-name "kolme-managed-signer-primary" \
  --signer-profile "ops-primary" \
  --signer-key-source "managed-external" \
  --sample-count 100 \
  --timeout-events 10 \
  --unavailable-events 0 \
  --error-events 8 \
  --max-timeout-rate-bps 500 \
  --max-unavailable-rate-bps 500 \
  --max-error-rate-bps 500 \
  --ci-fast-gate FAIL >/dev/null

set +e
no_go_output="$(
  python3 "$CHECKER" \
    --telemetry-bundle "$NO_GO_BUNDLE" \
    --output-json "$NO_GO_REPORT" 2>&1
)"
no_go_code=$?
set -e

if [ "$no_go_code" -eq 0 ]; then
  echo "expected NO-GO policy case to fail closed" >&2
  exit 1
fi
if [ "$(extract_value "$no_go_output" "status")" != "fail" ]; then
  echo "expected NO-GO policy status to be fail" >&2
  exit 1
fi
if [ "$(extract_value "$no_go_output" "final_decision")" != "NO-GO" ]; then
  echo "expected NO-GO policy final_decision=NO-GO" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q "managed_signer_backend_timeout_rate_threshold_exceeded"; then
  echo "expected timeout-rate threshold reason code in NO-GO policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q "managed_signer_backend_error_rate_threshold_exceeded"; then
  echo "expected error-rate threshold reason code in NO-GO policy output" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q "managed_signer_backend_ci_fast_gate_failed"; then
  echo "expected ci-fast-gate reason code in NO-GO policy output" >&2
  exit 1
fi

python3 - "$GO_REPORT" "$NO_GO_REPORT" <<'PY'
import json
import pathlib
import sys

go_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if go_payload.get("schema_version") != "kamn.kolme.managed-signer-backend-slo-policy-report.v1":
    raise SystemExit("unexpected policy report schema in GO report")
if go_payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final_decision in GO report")
if go_payload.get("reason_codes") != ["managed_signer_backend_slo_within_threshold"]:
    raise SystemExit("expected deterministic GO reason_codes list")
if go_payload.get("remediation_markers") != ["managed_signer_backend_no_action_required"]:
    raise SystemExit("expected deterministic GO remediation marker list")

no_go_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if no_go_payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected NO-GO final_decision in NO-GO report")
reasons = set(no_go_payload.get("reason_codes", []))
required_reasons = {
    "managed_signer_backend_timeout_rate_threshold_exceeded",
    "managed_signer_backend_error_rate_threshold_exceeded",
    "managed_signer_backend_ci_fast_gate_failed",
}
if not required_reasons.issubset(reasons):
    raise SystemExit("missing deterministic NO-GO threshold reason codes in report")
markers = set(no_go_payload.get("remediation_markers", []))
required_markers = {
    "managed_signer_backend_reduce_timeout_burst",
    "managed_signer_backend_enable_circuit_breaker",
    "managed_signer_backend_replay_ci_fast_gate",
}
if not required_markers.issubset(markers):
    raise SystemExit("missing deterministic NO-GO remediation markers in report")
PY

python3 - "$GO_BUNDLE" "$MALFORMED_BUNDLE" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
payload["signer_key_source"] = "raw-private-key"
pathlib.Path(sys.argv[2]).write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
malformed_output="$(
  python3 "$CHECKER" \
    --telemetry-bundle "$MALFORMED_BUNDLE" \
    --output-json "$MALFORMED_REPORT" 2>&1
)"
malformed_code=$?
set -e

if [ "$malformed_code" -eq 0 ]; then
  echo "expected malformed managed-signer telemetry bundle to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$malformed_output" | grep -q "signer_key_source_invalid"; then
  echo "expected signer_key_source_invalid reason code in malformed policy output" >&2
  exit 1
fi

echo "managed-signer backend SLO policy checker tests passed."
