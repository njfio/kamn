#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_signal_quarantine_evidence_bundle.sh"
CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_signal_quarantine_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected reputation signal quarantine evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected reputation signal quarantine policy checker to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-signal-quarantine-go.json"
bash "$GENERATOR" \
  --output-file "$bundle_file" \
  --lane contract \
  --signal-id "signal-go-policy-001" \
  --subject-did "did:kamn:agent-go-policy-001" \
  --signal-kind "ENDORSEMENT" \
  --source-channel "TELEGRAM" \
  --event-age-seconds 35 \
  --payload-sha256 "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  --payload-signature-verified PASS \
  --nonce-unique true \
  --rate-within-threshold true \
  --source-attested true \
  --ci-fast-gate PASS >/dev/null

tampered_bundle="$TMP_DIR/reputation-signal-quarantine-tampered-reason-codes.json"
cp "$bundle_file" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["reason_codes"] = ["event_stale"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered reason_codes payload to fail signal quarantine policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "reason_codes mismatch"; then
  echo "expected reason_codes mismatch failure for tampered signal quarantine payload" >&2
  exit 1
fi

# Regression: #935
if ! printf '%s\n' "$tampered_output" | grep -q "expected reason_codes"; then
  echo "expected explicit reason code mismatch output for signal quarantine regression path" >&2
  exit 1
fi

echo "reputation signal quarantine reason-code policy checker tests passed."
