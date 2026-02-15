#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_block_reconciliation_partition_rejoin_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
TMP_TAMPERED_TRANSPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_TAMPERED_TRANSPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected block reconciliation partition/rejoin validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected block reconciliation partition/rejoin policy checker script to be executable" >&2
  exit 1
fi

bash "$VALIDATION_SCRIPT" --mode dry-run --ci-fast-gate PASS --output-json "$TMP_REPORT" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected block reconciliation partition/rejoin policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected block reconciliation partition/rejoin policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^block_reconciliation_partition_rejoin_policy_status=verified$'; then
  echo "expected block reconciliation partition/rejoin policy checker status marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.block-reconciliation-partition-rejoin-live-policy-report.v1":
    raise SystemExit("unexpected block reconciliation partition/rejoin policy schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected block reconciliation partition/rejoin policy final_decision=GO")
if payload.get("block_reconciliation_partition_rejoin_policy_status") != "verified":
    raise SystemExit("expected block_reconciliation_partition_rejoin_policy_status=verified")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fast_gate_exclusion_status"] = "mismatch-marker"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered block reconciliation partition/rejoin report" >&2
  exit 1
fi

python3 - "$tampered_output" <<'PY'
import sys

output = sys.argv[1]
failed_checks = ""
for line in output.splitlines():
    if line.startswith("failed_checks="):
        failed_checks = line.split("=", 1)[1]
        break
reason_codes = [entry for entry in failed_checks.split(",") if entry]
if "block_reconciliation_partition_rejoin_policy_fast_gate_exclusion_mismatch" not in reason_codes:
    raise SystemExit("expected parser to recover deterministic block reconciliation partition/rejoin reason code")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED_TRANSPORT"
python3 - "$TMP_TAMPERED_TRANSPORT" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_transport_mode"] = "in_memory_simulation"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_transport_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_TRANSPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_transport_code=$?
set -e
if [ "$tampered_transport_code" -eq 0 ]; then
  echo "expected transport-mode tampered block reconciliation partition/rejoin report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_transport_output" | grep -q 'block_reconciliation_partition_rejoin_policy_transport_mode_mismatch'; then
  echo "expected deterministic transport-mode mismatch reason for block reconciliation partition/rejoin report" >&2
  exit 1
fi

echo "block reconciliation partition/rejoin live policy tests passed."
