#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/runtime/service_api_tranche1_wrapper_family_parity_contract.py"
MATRIX_FILE="$ROOT_DIR/fixtures/ci/service_api_tranche1_wrapper_family_matrix.json"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected service api tranche-1 wrapper parity checker to be executable" >&2
  exit 1
fi
if [ ! -f "$MATRIX_FILE" ]; then
  echo "expected service api tranche-1 wrapper family matrix file" >&2
  exit 1
fi

parity_output="$(
  python3 "$CHECKER" \
    --root-dir "$ROOT_DIR" \
    --matrix-file "$MATRIX_FILE"
)"
if ! printf '%s\n' "$parity_output" | grep -q '^status=pass$'; then
  echo "expected service api tranche-1 parity checker status=pass" >&2
  exit 1
fi
if ! printf '%s\n' "$parity_output" | grep -q '^service_api_tranche1_wrapper_family_status=verified$'; then
  echo "expected service api tranche-1 parity checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$parity_output" | grep -q '^reason_codes=none$'; then
  echo "expected service api tranche-1 parity checker reason code marker" >&2
  exit 1
fi

while IFS=$'\t' read -r wrapper contract_key policy_key tamper_reason; do
  lane_output="$(
    bash "$ROOT_DIR/$wrapper" \
      --mode dry-run \
      --output-json "$TMP_DIR/$(basename "$wrapper" .sh)-report.json" \
      --policy-output-json "$TMP_DIR/$(basename "$wrapper" .sh)-policy.json"
  )"
  if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
    echo "expected wrapper lane status marker for $wrapper" >&2
    exit 1
  fi
  if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
    echo "expected wrapper lane final decision marker for $wrapper" >&2
    exit 1
  fi
  if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
    echo "expected wrapper lane mode marker for $wrapper" >&2
    exit 1
  fi
  if ! printf '%s\n' "$lane_output" | grep -q "^${contract_key}=verified$"; then
    echo "expected wrapper lane contract status marker for $wrapper" >&2
    exit 1
  fi
  if ! printf '%s\n' "$lane_output" | grep -q "^${policy_key}=verified$"; then
    echo "expected wrapper lane policy status marker for $wrapper" >&2
    exit 1
  fi
  if ! printf '%s\n' "$lane_output" | grep -q "^fail_closed_reason_code=${tamper_reason}$"; then
    echo "expected wrapper lane fail-closed reason marker for $wrapper" >&2
    exit 1
  fi
done < <(
  python3 - "$MATRIX_FILE" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
for wrapper in payload["wrappers"]:
    print(
        "\t".join(
            [
                wrapper["wrapper"],
                wrapper["contract_status_key"],
                wrapper["policy_status_key"],
                wrapper["tamper_reason_code"],
            ]
        )
    )
PY
)

if ! grep -q "test_service_api_tranche1_wrapper_family_parity_matrix.sh" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to reference service api tranche-1 parity matrix command" >&2
  exit 1
fi
if ! grep -q "service api tranche-1 runner migration parity guard" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include service api tranche-1 migration marker" >&2
  exit 1
fi

tampered_matrix="$TMP_DIR/service-api-tranche1-wrapper-family-matrix.tampered.json"
cp "$MATRIX_FILE" "$tampered_matrix"
python3 - "$tampered_matrix" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["wrappers"][0]["policy_checker"] = "scripts/runtime/check_service_api_prometheus_metrics_live_policy_drifted.sh"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  python3 "$CHECKER" \
    --root-dir "$ROOT_DIR" \
    --matrix-file "$tampered_matrix" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered service api tranche-1 matrix to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'wrapper_policy_checker_marker_missing:scripts/runtime/validate_service_api_prometheus_metrics_live_contract_lane.sh'; then
  echo "expected deterministic policy-checker drift reason code for tampered service api tranche-1 matrix" >&2
  exit 1
fi

echo "service api tranche-1 wrapper family parity matrix tests passed."
