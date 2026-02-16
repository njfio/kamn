#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/sdk/validate_rust_sdk_service_client_live.sh"
RUST_SDK_DOC="$ROOT_DIR/docs/sdk/rust-sdk.md"
SDK_README_DOC="$ROOT_DIR/docs/sdk/README.md"
REQUEST_ERROR_REASON_TAXONOMY_VERSION="kamn.sdk.rust-http-request-error-reason-taxonomy.v1"
REQUEST_ERROR_REASON_CODES_CSV="service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_route_not_found,service_api_method_not_allowed,service_api_legacy_unauthorized,service_api_legacy_conflict,service_api_legacy_bad_request,service_api_legacy_error_unknown"
SUBSCRIPTION_REASON_TAXONOMY_VERSION="kamn.sdk.websocket-subscription-reason-taxonomy.v1"
SUBSCRIPTION_REASON_CODES_CSV="service_api_websocket_upgrade_required,service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_route_not_found,service_api_method_not_allowed"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected rust sdk service client live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected rust sdk service client live pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected rust sdk service client live GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^service_client_contract_status=verified$'; then
  echo "expected rust sdk service client live contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^evidence_bundle_status=verified$'; then
  echo "expected rust sdk service client live evidence marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected rust sdk service client live fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_reason_code=zero_runtime_budget$'; then
  echo "expected rust sdk service client live fail-closed reason marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^http_error_taxonomy_contract_status=verified$'; then
  echo "expected rust sdk service client live http error taxonomy contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^request_error_reason_taxonomy_version=${REQUEST_ERROR_REASON_TAXONOMY_VERSION}$"; then
  echo "expected rust sdk service client live request-error taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^request_error_reason_codes_csv=${REQUEST_ERROR_REASON_CODES_CSV}$"; then
  echo "expected rust sdk service client live request-error reason-codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^subscription_contract_status=verified$'; then
  echo "expected rust sdk service client live subscription contract marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^subscription_reason_taxonomy_version=${SUBSCRIPTION_REASON_TAXONOMY_VERSION}$"; then
  echo "expected rust sdk service client live subscription taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^subscription_reason_codes_csv=${SUBSCRIPTION_REASON_CODES_CSV}$"; then
  echo "expected rust sdk service client live subscription reason-codes marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.rust-service-client-live-validation.v1":
    raise SystemExit("unexpected rust sdk service client live schema")
if payload.get("status") != "pass":
    raise SystemExit("expected live status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live final_decision=GO")
if payload.get("service_client_contract_status") != "verified":
    raise SystemExit("expected service_client_contract_status=verified")
if payload.get("evidence_bundle_status") != "verified":
    raise SystemExit("expected evidence_bundle_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("fail_closed_reason_code") != "zero_runtime_budget":
    raise SystemExit("expected fail_closed_reason_code=zero_runtime_budget")
if payload.get("http_error_taxonomy_contract_status") != "verified":
    raise SystemExit("expected http_error_taxonomy_contract_status=verified")
if payload.get("request_error_reason_taxonomy_version") != "kamn.sdk.rust-http-request-error-reason-taxonomy.v1":
    raise SystemExit("expected request_error_reason_taxonomy_version marker")
if payload.get("request_error_reason_codes_csv") != "service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_route_not_found,service_api_method_not_allowed,service_api_legacy_unauthorized,service_api_legacy_conflict,service_api_legacy_bad_request,service_api_legacy_error_unknown":
    raise SystemExit("expected request_error_reason_codes_csv marker")
if payload.get("subscription_contract_status") != "verified":
    raise SystemExit("expected subscription_contract_status=verified")
if payload.get("subscription_reason_taxonomy_version") != "kamn.sdk.websocket-subscription-reason-taxonomy.v1":
    raise SystemExit("expected subscription_reason_taxonomy_version marker")
if payload.get("subscription_reason_codes_csv") != "service_api_websocket_upgrade_required,service_api_auth_sender_did_header_missing,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected,service_api_route_not_found,service_api_method_not_allowed":
    raise SystemExit("expected subscription_reason_codes_csv marker")
PY

set +e
invalid_budget_output="$({ bash "$VALIDATION_SCRIPT" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected rust sdk service client live script to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

set +e
zero_budget_output="$({ bash "$VALIDATION_SCRIPT" --max-seconds 0; } 2>&1)"
zero_budget_code=$?
set -e
if [ "$zero_budget_code" -eq 0 ]; then
  echo "expected rust sdk service client live script to reject zero max-seconds" >&2
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
if ! grep -Fq "subscription_reason_taxonomy_version=${SUBSCRIPTION_REASON_TAXONOMY_VERSION}" "$RUST_SDK_DOC"; then
  echo "expected rust sdk docs to include subscription taxonomy version marker" >&2
  exit 1
fi
if [ ! -f "$SDK_README_DOC" ]; then
  echo "expected sdk README docs file for request-error taxonomy marker checks" >&2
  exit 1
fi
if ! grep -Fq "request_error_reason_codes_csv=${REQUEST_ERROR_REASON_CODES_CSV}" "$SDK_README_DOC"; then
  echo "expected sdk README to include request-error reason-codes marker" >&2
  exit 1
fi
if ! grep -Fq "subscription_reason_codes_csv=${SUBSCRIPTION_REASON_CODES_CSV}" "$SDK_README_DOC"; then
  echo "expected sdk README to include subscription reason-codes marker" >&2
  exit 1
fi

echo "rust sdk service client live validation tests passed."
