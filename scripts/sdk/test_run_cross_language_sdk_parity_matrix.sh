#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_cross_language_sdk_parity_matrix.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected cross-language sdk parity matrix runner to be executable" >&2
  exit 1
fi

FIXTURE="$TMP_DIR/parity-fixture.json"
cat >"$FIXTURE" <<'JSON'
{
  "cases": [
    {
      "id": "baseline-ok",
      "agent_type": "autonomous",
      "model_family": "claude-4",
      "capabilities": ["text"],
      "expected": {
        "status": "ok"
      }
    }
  ]
}
JSON

REPORT="$TMP_DIR/parity-matrix-report.json"
run_output="$({
  bash "$RUNNER" \
    --mode contract \
    --languages python \
    --fixture "$FIXTURE" \
    --max-seconds 180 \
    --output-json "$REPORT"
} 2>&1)"

if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected cross-language sdk parity matrix pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected cross-language sdk parity matrix GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^register_parity_status=verified$'; then
  echo "expected register parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^live_transport_parity_status=verified$'; then
  echo "expected live transport parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^mode=contract$'; then
  echo "expected contract mode marker" >&2
  exit 1
fi

python3 - "$REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.sdk.cross-language-parity.v1":
    raise SystemExit("unexpected cross-language parity report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected report status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected report final_decision=GO")
if payload.get("mode") != "contract":
    raise SystemExit("expected report mode=contract")
if payload.get("register_parity_status") != "verified":
    raise SystemExit("expected register_parity_status=verified")
if payload.get("live_transport_parity_status") != "verified":
    raise SystemExit("expected live_transport_parity_status=verified")
PY

set +e
invalid_mode_output="$({ bash "$RUNNER" --mode invalid; } 2>&1)"
invalid_mode_code=$?
set -e
if [ "$invalid_mode_code" -eq 0 ]; then
  echo "expected runner to fail for invalid mode" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_mode_output" | grep -q 'mode must be one of: contract,deep'; then
  echo "expected deterministic invalid mode marker" >&2
  exit 1
fi

echo "cross-language sdk parity matrix runner tests passed."
