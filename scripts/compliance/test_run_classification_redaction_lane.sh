#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
LANE_SCRIPT="$ROOT_DIR/scripts/compliance/run_classification_redaction_lane.sh"
LANE_IMPL="$ROOT_DIR/scripts/compliance/run_classification_redaction_lane_impl.sh"
SHARED_LANE="$ROOT_DIR/scripts/compliance/classification_redaction_lane_contract.py"
DISPATCHER="$ROOT_DIR/scripts/framework/run_non_kolme_contract_lane_dispatch.sh"
MANIFEST_FILE="$ROOT_DIR/scripts/framework/manifests/compliance_classification_redaction_lane.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

test_harness_require_executable "$LANE_SCRIPT" "expected classification/redaction lane script to be executable"
test_harness_require_executable "$LANE_IMPL" "expected classification/redaction lane implementation to be executable"
test_harness_require_executable "$DISPATCHER" "expected shared non-Kolme dispatcher to be executable"

if [ ! -L "$LANE_SCRIPT" ]; then
  echo "expected classification/redaction lane wrapper to be a dispatcher symlink" >&2
  exit 1
fi
if [ "$(readlink "$LANE_SCRIPT")" != "../framework/run_non_kolme_contract_lane_dispatch.sh" ]; then
  echo "expected classification/redaction lane wrapper to target shared non-Kolme dispatcher" >&2
  exit 1
fi
resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$LANE_SCRIPT")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST_FILE" ]; then
  echo "expected classification/redaction lane wrapper to resolve compliance manifest via dispatcher" >&2
  exit 1
fi
if ! grep -q 'run_classification_redaction_lane_impl.sh' "$MANIFEST_FILE"; then
  echo "expected classification/redaction lane manifest to dispatch implementation module" >&2
  exit 1
fi

test_harness_require_executable "$SHARED_LANE" "expected shared classification/redaction lane implementation to be executable"

go_report="$TMP_DIR/classification-redaction-go.json"
go_output="$(
  KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS=true \
    bash "$LANE_SCRIPT" --output-file "$go_report"
)"
if [ "$(extract_value "$go_output" "status")" != "ok" ]; then
  echo "expected classification/redaction lane GO path status=ok" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "final_decision")" != "GO" ]; then
  echo "expected classification/redaction lane GO path final_decision=GO" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "reason_key")" != "classification_redaction_reason_codes:GO:v1" ]; then
  echo "expected classification/redaction lane GO path reason_key marker" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.compliance.classification-redaction-report.v1"' "$go_report"; then
  echo "expected classification/redaction lane report schema marker" >&2
  exit 1
fi

no_go_report="$TMP_DIR/classification-redaction-no-go.json"
no_go_output="$(
  KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS=true \
  KAMN_CLASSIFICATION_REDACTION_FORCE_REDACTION_MISSING=true \
    bash "$LANE_SCRIPT" --output-file "$no_go_report"
)"
if [ "$(extract_value "$no_go_output" "final_decision")" != "NO-GO" ]; then
  echo "expected classification/redaction lane forced missing redaction path final_decision=NO-GO" >&2
  exit 1
fi
if [ "$(extract_value "$no_go_output" "reason_key")" != "classification_redaction_reason_codes:NO-GO:v1" ]; then
  echo "expected classification/redaction lane forced missing redaction path reason_key marker" >&2
  exit 1
fi

if ! grep -q '"redaction_contract_missing"' "$no_go_report"; then
  echo "expected forced redaction-missing path to emit redaction_contract_missing reason" >&2
  exit 1
fi

echo "classification/redaction lane script tests passed."
