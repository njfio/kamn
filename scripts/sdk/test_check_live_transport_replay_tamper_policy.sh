#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/sdk/generate_live_transport_replay_tamper_evidence_bundle.sh"
CHECKER="$ROOT_DIR/scripts/sdk/check_live_transport_replay_tamper_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

if [ ! -x "$GENERATOR" ]; then
  echo "expected live transport replay/tamper evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected live transport replay/tamper policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/live-transport-replay-tamper-go.json"
bash "$GENERATOR" \
  --output-file "$go_bundle" \
  --transport-lane-id "localhost-signed-integration" \
  --message-id "msg-go-001" \
  --from-did "kamn:did:agent:sender-1" \
  --to-did "kamn:did:agent:listener-1" \
  --nonce 41 \
  --signature-status valid \
  --replay-detected false \
  --tamper-detected false \
  --ci-fast-gate PASS >/dev/null

go_output="$(bash "$CHECKER" --bundle-file "$go_bundle")"
if [ "$(extract_value "$go_output" "status")" != "ok" ]; then
  echo "expected replay/tamper policy checker to pass for GO bundle" >&2
  exit 1
fi
if [ "$(extract_value "$go_output" "final_decision")" != "GO" ]; then
  echo "expected replay/tamper policy checker to return GO for GO bundle" >&2
  exit 1
fi

tampered_bundle="$TMP_DIR/live-transport-replay-tamper-go.tampered.json"
cp "$go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered replay/tamper evidence bundle to fail policy checker" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected policy decision mismatch error for tampered final decision" >&2
  exit 1
fi

echo "live transport replay/tamper policy checker tests passed."
