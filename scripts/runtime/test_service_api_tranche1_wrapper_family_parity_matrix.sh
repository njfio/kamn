#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/runtime/service_api_tranche1_wrapper_family_parity_contract.py"
DISPATCHER="$ROOT_DIR/scripts/runtime/run_service_api_tranche2_contract_lane_dispatch.sh"
MATRIX_FILE="$ROOT_DIR/fixtures/ci/service_api_tranche1_wrapper_family_matrix.json"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected service api tranche-2 wrapper parity checker to be executable" >&2
  exit 1
fi
if [ ! -x "$DISPATCHER" ]; then
  echo "expected service api tranche-2 dispatcher script to be executable" >&2
  exit 1
fi
if [ ! -f "$MATRIX_FILE" ]; then
  echo "expected service api tranche-2 wrapper family matrix file" >&2
  exit 1
fi

parity_output="$(
  python3 "$CHECKER" \
    --root-dir "$ROOT_DIR" \
    --matrix-file "$MATRIX_FILE" \
    --output-json "$TMP_DIR/service-api-tranche2-wrapper-family-parity.report.json"
)"
if ! printf '%s\n' "$parity_output" | grep -q '^status=pass$'; then
  echo "expected service api tranche-2 parity checker status=pass" >&2
  exit 1
fi
if ! printf '%s\n' "$parity_output" | grep -q '^service_api_tranche2_wrapper_family_status=verified$'; then
  echo "expected service api tranche-2 parity checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$parity_output" | grep -q '^reason_codes=none$'; then
  echo "expected service api tranche-2 parity checker reason code marker" >&2
  exit 1
fi
if ! printf '%s\n' "$parity_output" | grep -q '^reason_taxonomy_version=kamn.runtime.service-api-tranche2-wrapper-family-parity-reason-taxonomy.v1$'; then
  echo "expected service api tranche-2 parity checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$parity_output" | grep -q '^reason_codes_csv=impl_contract_status_marker_missing,impl_missing,impl_not_executable,impl_policy_checker_marker_missing,impl_policy_status_marker_missing,impl_runner_entry_marker_missing,impl_runner_source_marker_missing,impl_tamper_reason_marker_missing,impl_validation_script_marker_missing,matrix_wrapper_entry_invalid,service_api_tranche2_impl_shell_loc_budget_exceeded,service_api_tranche2_wrapper_shell_loc_budget_exceeded,wrapper_dispatch_target_mismatch,wrapper_missing,wrapper_not_symlink$'; then
  echo "expected service api tranche-2 parity checker reason taxonomy codes marker" >&2
  exit 1
fi

python3 - "$TMP_DIR/service-api-tranche2-wrapper-family-parity.report.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-tranche2-wrapper-family-parity-report.v1":
    raise SystemExit("expected deterministic schema version for service api tranche-2 parity report")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass in service api tranche-2 parity report")
if payload.get("service_api_tranche2_wrapper_family_status") != "verified":
    raise SystemExit("expected verified wrapper family status in service api tranche-2 parity report")
if payload.get("reason_taxonomy_version") != "kamn.runtime.service-api-tranche2-wrapper-family-parity-reason-taxonomy.v1":
    raise SystemExit("expected reason taxonomy version in service api tranche-2 parity report")
if payload.get("reason_codes_csv") != "impl_contract_status_marker_missing,impl_missing,impl_not_executable,impl_policy_checker_marker_missing,impl_policy_status_marker_missing,impl_runner_entry_marker_missing,impl_runner_source_marker_missing,impl_tamper_reason_marker_missing,impl_validation_script_marker_missing,matrix_wrapper_entry_invalid,service_api_tranche2_impl_shell_loc_budget_exceeded,service_api_tranche2_wrapper_shell_loc_budget_exceeded,wrapper_dispatch_target_mismatch,wrapper_missing,wrapper_not_symlink":
    raise SystemExit("expected deterministic reason code taxonomy set in service api tranche-2 parity report")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes list in service api tranche-2 parity report")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none in service api tranche-2 parity report")
PY

while IFS=$'\t' read -r wrapper impl contract_key policy_key tamper_reason; do
  wrapper_path="$ROOT_DIR/$wrapper"
  if [ ! -L "$wrapper_path" ]; then
    echo "expected service api tranche-2 wrapper to be a dispatcher symlink: $wrapper" >&2
    exit 1
  fi
  if [ "$(readlink "$wrapper_path")" != "run_service_api_tranche2_contract_lane_dispatch.sh" ]; then
    echo "expected service api tranche-2 wrapper to target dispatcher script: $wrapper" >&2
    exit 1
  fi

  resolved_impl_path="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$wrapper")" --resolve-impl-path)"
  expected_impl_path="$ROOT_DIR/$impl"
  if [ "$resolved_impl_path" != "$expected_impl_path" ]; then
    echo "expected service api tranche-2 dispatcher to resolve implementation for $wrapper" >&2
    exit 1
  fi

  lane_output="$(
    bash "$wrapper_path" \
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
                wrapper["impl_script"],
                wrapper["contract_status_key"],
                wrapper["policy_status_key"],
                wrapper["tamper_reason_code"],
            ]
        )
    )
PY
)

if ! grep -q "test_service_api_tranche1_wrapper_family_parity_matrix.sh" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to reference service api tranche-2 parity matrix command" >&2
  exit 1
fi
if ! grep -q "service api tranche-2 wrapper retirement parity guard" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include service api tranche-2 migration marker" >&2
  exit 1
fi

set +e
unknown_wrapper_output="$(bash "$DISPATCHER" --lane-wrapper validate_service_api_unknown_contract_lane.sh --resolve-impl-path 2>&1)"
unknown_wrapper_code=$?
set -e
if [ "$unknown_wrapper_code" -eq 0 ]; then
  echo "expected service api tranche-2 dispatcher to fail for unknown wrapper" >&2
  exit 1
fi
if ! printf '%s\n' "$unknown_wrapper_output" | grep -q 'unknown service api tranche-2 wrapper for dispatch'; then
  echo "expected deterministic unknown-wrapper reason marker for service api tranche-2 dispatcher" >&2
  exit 1
fi

tampered_matrix="$TMP_DIR/service-api-tranche2-wrapper-family-matrix.tampered.json"
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
    --matrix-file "$tampered_matrix" \
    --output-json "$TMP_DIR/service-api-tranche2-wrapper-family-parity.tampered.report.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered service api tranche-2 matrix to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'impl_policy_checker_marker_missing:scripts/runtime/validate_service_api_prometheus_metrics_live_contract_lane_impl.sh'; then
  echo "expected deterministic policy-checker drift reason code for tampered service api tranche-2 matrix" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_taxonomy_version=kamn.runtime.service-api-tranche2-wrapper-family-parity-reason-taxonomy.v1$'; then
  echo "expected tampered service api tranche-2 matrix output to emit reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_csv=impl_contract_status_marker_missing,impl_missing,impl_not_executable,impl_policy_checker_marker_missing,impl_policy_status_marker_missing,impl_runner_entry_marker_missing,impl_runner_source_marker_missing,impl_tamper_reason_marker_missing,impl_validation_script_marker_missing,matrix_wrapper_entry_invalid,service_api_tranche2_impl_shell_loc_budget_exceeded,service_api_tranche2_wrapper_shell_loc_budget_exceeded,wrapper_dispatch_target_mismatch,wrapper_missing,wrapper_not_symlink$'; then
  echo "expected tampered service api tranche-2 matrix output to emit deterministic taxonomy code set" >&2
  exit 1
fi

python3 - "$TMP_DIR/service-api-tranche2-wrapper-family-parity.tampered.report.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected status=fail in tampered service api tranche-2 parity report")
if payload.get("service_api_tranche2_wrapper_family_status") != "rejected":
    raise SystemExit("expected rejected wrapper family status in tampered service api tranche-2 parity report")
if payload.get("reason_taxonomy_version") != "kamn.runtime.service-api-tranche2-wrapper-family-parity-reason-taxonomy.v1":
    raise SystemExit("expected reason taxonomy version in tampered service api tranche-2 parity report")
reason_codes = payload.get("reason_codes")
if not isinstance(reason_codes, list):
    raise SystemExit("expected reason_codes list in tampered service api tranche-2 parity report")
if "impl_policy_checker_marker_missing:scripts/runtime/validate_service_api_prometheus_metrics_live_contract_lane_impl.sh" not in reason_codes:
    raise SystemExit("expected tampered service api tranche-2 parity report to include policy checker drift reason code")
if payload.get("reason_codes_value") != "impl_policy_checker_marker_missing:scripts/runtime/validate_service_api_prometheus_metrics_live_contract_lane_impl.sh":
    raise SystemExit("expected tampered service api tranche-2 parity report normalized reason_codes_value marker")
PY

tampered_second_wrapper_matrix="$TMP_DIR/service-api-tranche2-wrapper-family-matrix.second-wrapper.tampered.json"
cp "$MATRIX_FILE" "$tampered_second_wrapper_matrix"
python3 - "$tampered_second_wrapper_matrix" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["wrappers"][1]["contract_status_key"] = "service_api_graceful_shutdown_drain_contract_status_drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_second_wrapper_output="$(
  python3 "$CHECKER" \
    --root-dir "$ROOT_DIR" \
    --matrix-file "$tampered_second_wrapper_matrix" 2>&1
)"
tampered_second_wrapper_code=$?
set -e

if [ "$tampered_second_wrapper_code" -eq 0 ]; then
  echo "expected second-wrapper tampered service api tranche-2 matrix to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_second_wrapper_output" | grep -q 'impl_contract_status_marker_missing:scripts/runtime/validate_service_api_graceful_shutdown_drain_live_contract_lane_impl.sh'; then
  echo "expected deterministic second-wrapper contract-status drift reason code for tampered service api tranche-2 matrix" >&2
  exit 1
fi

echo "service api tranche-2 wrapper family parity matrix tests passed."
