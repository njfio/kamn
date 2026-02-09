#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_dispute_evidence_bundle.sh"
CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_dispute_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected reputation dispute evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected reputation dispute policy checker to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-dispute-go.json"
bash "$GENERATOR" \
  --output-file "$bundle_file" \
  --dispute-id "dispute-go-reason-code-001" \
  --subject-did "did:kamn:agent-go-001" \
  --reviewer-did "did:kamn:reviewer-go-001" \
  --dispute-reason-code "QUALITY" \
  --evidence-uri "s3://kamn-audit/reputation/dispute-go-reason-code-001.json" \
  --evidence-sha256 "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
  --evidence-hash-verified "PASS" \
  --original-trust-score 650 \
  --proposed-trust-score 600 \
  --max-adjustment-points 90 \
  --policy-window-open true \
  --approval-recorded true \
  --ci-fast-gate PASS >/dev/null

tampered_bundle="$TMP_DIR/reputation-dispute-tampered-reason-codes.json"
cp "$bundle_file" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["reason_codes"] = ["policy_window_closed"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered reason_codes payload to fail dispute policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "reason_codes mismatch"; then
  echo "expected reason_codes mismatch failure for tampered dispute policy payload" >&2
  exit 1
fi

# Regression: #934
if ! printf '%s\n' "$tampered_output" | grep -q "expected reason_codes"; then
  echo "expected explicit reason code mismatch output for dispute policy regression path" >&2
  exit 1
fi

echo "reputation dispute reason-code policy checker tests passed."
