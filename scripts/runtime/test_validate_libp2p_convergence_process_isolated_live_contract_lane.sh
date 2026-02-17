#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh"
RUNBOOK_DOC="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"

for required_exec in "$CONTRACT_LANE" "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected process-isolated convergence script to be executable: $required_exec" >&2
    exit 1
  fi
done
if [ ! -f "$RUNBOOK_DOC" ]; then
  echo "expected process-isolated convergence runbook doc to exist" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
lane_report="$TMP_DIR/libp2p-convergence-process-isolated-contract-lane-report.json"
policy_report="$TMP_DIR/libp2p-convergence-process-isolated-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --lane-profile smoke \
    --ci-fast-gate PASS \
    --max-seconds 180 \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected process-isolated convergence contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated convergence contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_mode=dry-run$'; then
  echo "expected process-isolated convergence contract lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^lane_profile=smoke$'; then
  echo "expected process-isolated convergence contract lane smoke profile marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_process_isolated_convergence_contract_status=verified$'; then
  echo "expected process-isolated convergence contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^libp2p_process_isolated_convergence_policy_status=verified$'; then
  echo "expected process-isolated convergence policy status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^convergence_reason_taxonomy_version=kamn.runtime.libp2p-convergence-reason-taxonomy.v1$'; then
  echo "expected process-isolated convergence contract lane reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^convergence_reason_codes_csv=fork_choice_stale_block_height$'; then
  echo "expected process-isolated convergence contract lane reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^transport_classification_normalization_status=verified$'; then
  echo "expected process-isolated convergence contract lane transport classification normalization marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fork_choice_stale_height_classification_status=verified$'; then
  echo "expected process-isolated convergence contract lane stale-height classification marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^finality_taxonomy_mapping_status=verified$'; then
  echo "expected process-isolated convergence contract lane finality taxonomy mapping status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^runbook_marker_parity_status=verified$'; then
  echo "expected process-isolated convergence contract lane runbook marker parity status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1$'; then
  echo "expected process-isolated convergence contract lane finality taxonomy runbook reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch$'; then
  echo "expected process-isolated convergence contract lane finality taxonomy runbook reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^finality_taxonomy_runbook_reason_code=none$'; then
  echo "expected process-isolated convergence contract lane finality taxonomy runbook reason code marker on GO path" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=libp2p_process_isolated_convergence_policy_marker_missing:no_shared_state_zero_delivery_status$'; then
  echo "expected process-isolated convergence fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))

if lane_payload.get("schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-contract-lane-report.v1":
    raise SystemExit("unexpected process-isolated convergence contract-lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected process-isolated convergence contract-lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated convergence contract-lane final_decision=GO")
if lane_payload.get("libp2p_process_isolated_convergence_contract_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_convergence_contract_status=verified")
if lane_payload.get("libp2p_process_isolated_convergence_policy_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_convergence_policy_status=verified")
if lane_payload.get("lane_profile") != "smoke":
    raise SystemExit("expected lane_profile=smoke in contract lane report")
if lane_payload.get("convergence_reason_taxonomy_version") != "kamn.runtime.libp2p-convergence-reason-taxonomy.v1":
    raise SystemExit("expected deterministic convergence_reason_taxonomy_version marker")
if lane_payload.get("convergence_reason_codes_csv") != "fork_choice_stale_block_height":
    raise SystemExit("expected deterministic convergence_reason_codes_csv marker")
if lane_payload.get("transport_classification_normalization_status") != "verified":
    raise SystemExit("expected transport_classification_normalization_status=verified")
if lane_payload.get("fork_choice_stale_height_classification_status") != "verified":
    raise SystemExit("expected fork_choice_stale_height_classification_status=verified")
if lane_payload.get("finality_taxonomy_mapping_status") != "verified":
    raise SystemExit("expected finality_taxonomy_mapping_status=verified")
if lane_payload.get("runbook_marker_parity_status") != "verified":
    raise SystemExit("expected runbook_marker_parity_status=verified")
if lane_payload.get("finality_taxonomy_runbook_reason_taxonomy_version") != "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_taxonomy_version marker")
if lane_payload.get("finality_taxonomy_runbook_reason_codes_csv") != "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_codes_csv marker")
if lane_payload.get("finality_taxonomy_runbook_reason_code") != "none":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_code marker on GO path")

if policy_payload.get("schema_version") != "kamn.runtime.libp2p-convergence-process-isolated-live-policy-report.v1":
    raise SystemExit("unexpected process-isolated convergence policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected process-isolated convergence policy final_decision=GO")
if policy_payload.get("libp2p_process_isolated_convergence_policy_status") != "verified":
    raise SystemExit("expected libp2p_process_isolated_convergence_policy_status=verified in policy report")
if policy_payload.get("convergence_reason_taxonomy_version") != "kamn.runtime.libp2p-convergence-reason-taxonomy.v1":
    raise SystemExit("expected deterministic convergence_reason_taxonomy_version marker in policy report")
if policy_payload.get("convergence_reason_codes_csv") != "fork_choice_stale_block_height":
    raise SystemExit("expected deterministic convergence_reason_codes_csv marker in policy report")
if policy_payload.get("transport_classification_normalization_status") != "verified":
    raise SystemExit("expected transport_classification_normalization_status=verified in policy report")
if policy_payload.get("fork_choice_stale_height_classification_status") != "verified":
    raise SystemExit("expected fork_choice_stale_height_classification_status=verified in policy report")
if policy_payload.get("finality_taxonomy_mapping_status") != "verified":
    raise SystemExit("expected finality_taxonomy_mapping_status=verified in policy report")
if policy_payload.get("runbook_marker_parity_status") != "verified":
    raise SystemExit("expected runbook_marker_parity_status=verified in policy report")
if policy_payload.get("finality_taxonomy_runbook_reason_taxonomy_version") != "kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_taxonomy_version marker in policy report")
if policy_payload.get("finality_taxonomy_runbook_reason_codes_csv") != "finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_codes_csv marker in policy report")
if policy_payload.get("finality_taxonomy_runbook_reason_code") != "none":
    raise SystemExit("expected deterministic finality_taxonomy_runbook_reason_code marker on GO path in policy report")
PY

set +e
invalid_gate_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --lane-profile smoke \
    --ci-fast-gate MAYBE \
    --max-seconds 120 2>&1
)"
invalid_gate_code=$?
set -e
if [ "$invalid_gate_code" -eq 0 ]; then
  echo "expected process-isolated convergence contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for process-isolated convergence contract lane" >&2
  exit 1
fi

set +e
fail_gate_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --lane-profile smoke \
    --ci-fast-gate FAIL \
    --max-seconds 120 2>&1
)"
fail_gate_code=$?
set -e
if [ "$fail_gate_code" -eq 0 ]; then
  echo "expected process-isolated convergence contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_gate_output" | grep -q 'libp2p_process_isolated_convergence_policy_ci_fast_gate_failed'; then
  echo "expected deterministic ci-fast-gate failure marker for process-isolated convergence contract lane" >&2
  exit 1
fi

set +e
invalid_profile_output="$(
  bash "$CONTRACT_LANE" \
    --mode dry-run \
    --lane-profile unknown \
    --ci-fast-gate PASS \
    --max-seconds 120 2>&1
)"
invalid_profile_code=$?
set -e
if [ "$invalid_profile_code" -eq 0 ]; then
  echo "expected process-isolated convergence contract lane to reject invalid lane-profile value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_profile_output" | grep -q 'lane-profile must be smoke or deep'; then
  echo "expected deterministic invalid lane-profile marker for process-isolated convergence contract lane" >&2
  exit 1
fi

set +e
deep_run_without_opt_in_output="$(
  bash "$CONTRACT_LANE" \
    --mode run \
    --lane-profile deep \
    --ci-fast-gate FAIL \
    --max-seconds 120 2>&1
)"
deep_run_without_opt_in_code=$?
set -e
if [ "$deep_run_without_opt_in_code" -eq 0 ]; then
  echo "expected deep run-mode contract lane without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$deep_run_without_opt_in_output" | grep -q 'KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_DEEP_OPT_IN=1'; then
  echo "expected deterministic deep-lane opt-in marker for contract lane run mode" >&2
  exit 1
fi

tampered_runbook_doc="$TMP_DIR/kolme_devnet_ops.runbook.tampered.md"
cp "$RUNBOOK_DOC" "$tampered_runbook_doc"
python3 - "$tampered_runbook_doc" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace(
    "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected",
)
path.write_text(text, encoding="utf-8")
PY

set +e
runbook_divergence_lane_output="$(
  KAMN_LIBP2P_CONVERGENCE_PROCESS_ISOLATED_RUNBOOK_DOC_OVERRIDE="$tampered_runbook_doc" \
    bash "$CONTRACT_LANE" \
      --mode dry-run \
      --lane-profile smoke \
      --ci-fast-gate PASS \
      --max-seconds 120 2>&1
)"
runbook_divergence_lane_code=$?
set -e
if [ "$runbook_divergence_lane_code" -eq 0 ]; then
  echo "expected process-isolated convergence contract lane to fail closed for runbook marker divergence" >&2
  exit 1
fi
if ! printf '%s\n' "$runbook_divergence_lane_output" | grep -q 'runbook_marker_parity_mismatch'; then
  echo "expected deterministic runbook marker parity mismatch reason for process-isolated convergence contract lane runbook divergence path" >&2
  exit 1
fi

echo "process-isolated libp2p convergence contract lane tests passed."
