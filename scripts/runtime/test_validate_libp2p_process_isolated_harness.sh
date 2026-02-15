#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_libp2p_process_isolated_harness.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected process-isolated harness validation script to be executable" >&2
  exit 1
fi

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode dry-run \
    --max-seconds 120 \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected process-isolated harness status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected process-isolated harness final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^two_node_startup_status=verified$'; then
  echo "expected process-isolated harness two-node startup marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^three_node_startup_status=verified$'; then
  echo "expected process-isolated harness three-node startup marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^partition_rejoin_status=verified$'; then
  echo "expected process-isolated harness partition/rejoin marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^publish_drop_recovery_status=verified$'; then
  echo "expected process-isolated harness publish-drop marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^runtime_transport_mode=libp2p_process_isolated_convergence$'; then
  echo "expected process-isolated harness runtime transport marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.libp2p-process-isolated-harness-report.v1":
    raise SystemExit("unexpected process-isolated harness report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("runtime_transport_mode") != "libp2p_process_isolated_convergence":
    raise SystemExit("expected runtime transport mode marker")
if payload.get("topology_node_counts") != [2, 3]:
    raise SystemExit("expected deterministic topology node counts")
reason_codes = payload.get("convergence_reason_codes")
if reason_codes != ["fork_choice_stale_block_height"]:
    raise SystemExit("expected deterministic convergence reason-code marker")
evidence_file = payload.get("process_harness_evidence_file")
if not evidence_file:
    raise SystemExit("expected process_harness_evidence_file marker")
if not pathlib.Path(evidence_file).is_file():
    raise SystemExit("expected process_harness_evidence_file to exist")
PY

set +e
run_without_opt_in_output="$({
  bash "$VALIDATION_SCRIPT" --mode run --max-seconds 120
} 2>&1)"
run_without_opt_in_code=$?
set -e
if [ "$run_without_opt_in_code" -eq 0 ]; then
  echo "expected run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$run_without_opt_in_output" | grep -q 'KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_OPT_IN=1'; then
  echo "expected deterministic opt-in marker for process-isolated harness run mode" >&2
  exit 1
fi

echo "process-isolated libp2p harness validation tests passed."
