#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_rust_sdk_service_client_contract.sh"
RUST_SDK_DOC="$ROOT_DIR/docs/sdk/rust-sdk.md"
SDK_README_DOC="$ROOT_DIR/docs/sdk/README.md"
REQUEST_ERROR_REASON_TAXONOMY_VERSION="kamn.sdk.rust-http-request-error-reason-taxonomy.v1"
REQUEST_ERROR_REASON_CODES_CSV="service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_route_not_found,service_api_method_not_allowed,service_api_legacy_unauthorized,service_api_legacy_conflict,service_api_legacy_bad_request,service_api_legacy_error_unknown"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected rust sdk service client contract runner to be executable" >&2
  exit 1
fi

run_output="$(bash "$RUNNER" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected rust sdk service client contract pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected rust sdk service client contract GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^http_route_contract_status=verified$'; then
  echo "expected rust sdk service client contract http marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^websocket_contract_status=verified$'; then
  echo "expected rust sdk service client contract websocket marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^regression_guard_status=verified$'; then
  echo "expected rust sdk service client contract regression marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q "^request_error_reason_taxonomy_version=${REQUEST_ERROR_REASON_TAXONOMY_VERSION}$"; then
  echo "expected rust sdk service client contract request-error taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q "^request_error_reason_codes_csv=${REQUEST_ERROR_REASON_CODES_CSV}$"; then
  echo "expected rust sdk service client contract request-error reason-codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^request_error_taxonomy_status=verified$'; then
  echo "expected rust sdk service client contract request-error taxonomy status marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.rust-service-client-contract.v1":
    raise SystemExit("unexpected rust sdk service client contract schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("http_route_contract_status") != "verified":
    raise SystemExit("expected http_route_contract_status=verified")
if payload.get("websocket_contract_status") != "verified":
    raise SystemExit("expected websocket_contract_status=verified")
if payload.get("regression_guard_status") != "verified":
    raise SystemExit("expected regression_guard_status=verified")
if payload.get("request_error_reason_taxonomy_version") != "kamn.sdk.rust-http-request-error-reason-taxonomy.v1":
    raise SystemExit("expected request_error_reason_taxonomy_version marker")
if payload.get("request_error_reason_codes_csv") != "service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_route_not_found,service_api_method_not_allowed,service_api_legacy_unauthorized,service_api_legacy_conflict,service_api_legacy_bad_request,service_api_legacy_error_unknown":
    raise SystemExit("expected request_error_reason_codes_csv marker")
if payload.get("request_error_taxonomy_status") != "verified":
    raise SystemExit("expected request_error_taxonomy_status=verified")
PY

set +e
invalid_budget_output="$({ bash "$RUNNER" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected rust sdk service client contract runner to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

set +e
zero_budget_output="$({ bash "$RUNNER" --max-seconds 0; } 2>&1)"
zero_budget_code=$?
set -e
if [ "$zero_budget_code" -eq 0 ]; then
  echo "expected rust sdk service client contract runner to reject zero max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$zero_budget_output" | grep -q 'max-seconds must be greater than zero'; then
  echo "expected deterministic zero max-seconds marker" >&2
  exit 1
fi

if [ ! -f "$RUST_SDK_DOC" ]; then
  echo "expected rust sdk docs file for request-error taxonomy marker checks" >&2
  exit 1
fi
if ! grep -Fq "request_error_reason_taxonomy_version=${REQUEST_ERROR_REASON_TAXONOMY_VERSION}" "$RUST_SDK_DOC"; then
  echo "expected rust sdk docs to include request-error taxonomy version marker" >&2
  exit 1
fi
if ! grep -Fq "request_error_reason_codes_csv=${REQUEST_ERROR_REASON_CODES_CSV}" "$RUST_SDK_DOC"; then
  echo "expected rust sdk docs to include request-error reason-codes marker" >&2
  exit 1
fi
if [ ! -f "$SDK_README_DOC" ]; then
  echo "expected sdk README docs file for request-error taxonomy marker checks" >&2
  exit 1
fi
if ! grep -Fq "request_error_reason_taxonomy_version=${REQUEST_ERROR_REASON_TAXONOMY_VERSION}" "$SDK_README_DOC"; then
  echo "expected sdk README to include request-error taxonomy version marker" >&2
  exit 1
fi

echo "rust sdk service client contract tests passed."
