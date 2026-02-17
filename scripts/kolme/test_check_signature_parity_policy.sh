#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/kolme/check_signature_parity_policy.py"
TMP_REPORT_GO="$(mktemp)"
TMP_REPORT_BAD="$(mktemp)"
TMP_REPORT_REASON_BAD="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_ERR="$(mktemp)"
trap 'rm -f "$TMP_REPORT_GO" "$TMP_REPORT_BAD" "$TMP_REPORT_REASON_BAD" "$TMP_POLICY" "$TMP_ERR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected signature parity policy checker to be executable" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_REPORT_GO" <<'JSON'
{
  "schema_version": "kamn.kolme.signature-parity-matrix-report.v1",
  "status": "pass",
  "fixture": "/tmp/signature-vectors.json",
  "source_contract": "njfio/kolme_fork-secp256k1-v1",
  "vector_count": 5,
  "failed_count": 0,
  "failed_vector_ids": [],
  "cases": [
    {
      "vector_id": "kolme_fork_primary_alpha",
      "observed_final_decision": "GO",
      "passed": true,
      "reason_codes": [],
      "missing_required_reason_codes": []
    },
    {
      "vector_id": "kolme_fork_secondary_beta",
      "observed_final_decision": "GO",
      "passed": true,
      "reason_codes": [],
      "missing_required_reason_codes": []
    },
    {
      "vector_id": "kolme_fork_primary_alpha_bad_signature",
      "observed_final_decision": "NO-GO",
      "passed": true,
      "reason_codes": [
        "parity_signature_mismatch"
      ],
      "missing_required_reason_codes": []
    },
    {
      "vector_id": "kolme_fork_secondary_beta_bad_recovery",
      "observed_final_decision": "NO-GO",
      "passed": true,
      "reason_codes": [
        "parity_recovery_id_mismatch"
      ],
      "missing_required_reason_codes": []
    },
    {
      "vector_id": "kolme_fork_primary_alpha_bad_pubkey",
      "observed_final_decision": "NO-GO",
      "passed": true,
      "reason_codes": [
        "parity_pubkey_mismatch"
      ],
      "missing_required_reason_codes": []
    }
  ]
}
JSON

python3 "$CHECKER" \
  --report-file "$TMP_REPORT_GO" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --require-vector-id kolme_fork_primary_alpha \
  --require-vector-id kolme_fork_secondary_beta \
  --require-vector-id kolme_fork_primary_alpha_bad_signature \
  --require-vector-id kolme_fork_secondary_beta_bad_recovery \
  --require-vector-id kolme_fork_primary_alpha_bad_pubkey \
  --output-json "$TMP_POLICY" >/dev/null

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.signature-parity-policy-report.v1":
    raise SystemExit("unexpected signature parity policy report schema")
if report.get("final_decision") != "GO":
    raise SystemExit("expected signature parity policy final_decision GO")
if report.get("reason_codes") != []:
    raise SystemExit("expected no signature parity policy reason codes for valid report")
PY

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_REPORT_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.signature-parity-matrix-report.v1",
  "status": "pass",
  "fixture": "/tmp/signature-vectors.json",
  "source_contract": "njfio/kolme_fork-secp256k1-v1",
  "vector_count": 0,
  "failed_count": 0,
  "failed_vector_ids": [],
  "cases": []
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_BAD" \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY" >"$TMP_ERR" 2>&1
bad_exit_code=$?
set -e

if [ "$bad_exit_code" -eq 0 ]; then
  echo "expected invalid signature parity report to fail closed" >&2
  exit 1
fi

if ! grep -q "vector_count_invalid" "$TMP_ERR"; then
  echo "expected vector_count_invalid reason in signature parity policy output" >&2
  exit 1
fi

if ! grep -q "cases_missing" "$TMP_ERR"; then
  echo "expected cases_missing reason in signature parity policy output" >&2
  exit 1
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_REPORT_REASON_BAD" <<'JSON'
{
  "schema_version": "kamn.kolme.signature-parity-matrix-report.v1",
  "status": "fail",
  "fixture": "/tmp/signature-vectors.json",
  "source_contract": "njfio/kolme_fork-secp256k1-v1",
  "vector_count": 1,
  "failed_count": 1,
  "failed_vector_ids": [
    "kolme_fork_primary_alpha_bad_signature"
  ],
  "cases": [
    {
      "vector_id": "kolme_fork_primary_alpha_bad_signature",
      "observed_final_decision": "NO-GO",
      "passed": false,
      "reason_codes": []
    }
  ]
}
JSON

set +e
python3 "$CHECKER" \
  --report-file "$TMP_REPORT_REASON_BAD" \
  --expected-final-decision NO-GO \
  --ci-fast-gate PASS \
  --output-json "$TMP_POLICY" >"$TMP_ERR" 2>&1
reason_bad_exit_code=$?
set -e

if [ "$reason_bad_exit_code" -eq 0 ]; then
  echo "expected parity policy checker to fail when NO-GO cases omit deterministic reason codes" >&2
  exit 1
fi

if ! grep -q "case_reason_codes_missing:kolme_fork_primary_alpha_bad_signature" "$TMP_ERR"; then
  echo "expected case_reason_codes_missing reason in signature parity policy output" >&2
  exit 1
fi

echo "signature parity policy checker tests passed."
