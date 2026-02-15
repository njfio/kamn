#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PLAN_DOC="$ROOT_DIR/docs/plans/2026-02-14-production-service-next-steps.md"
TMP_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_TAMPERED"' EXIT

if [ ! -f "$PLAN_DOC" ]; then
  echo "expected production-service next-steps plan document to exist" >&2
  exit 1
fi

python3 - "$PLAN_DOC" <<'PY'
from __future__ import annotations

import pathlib
import sys

doc_path = pathlib.Path(sys.argv[1])
text = doc_path.read_text(encoding="utf-8")

required_markers = (
    "Status Truth Snapshot",
    "| 1. HTTP ingress runtime | Delivered (`axum` server + auth/ws integration) |",
    "| 2. Persistent storage | Delivered (sqlite backend adapters + migration parity) |",
    "| 3. Real P2P transport | Open (libp2p transport hardening still active) |",
    "| 4. Transport-fed consensus pipeline | Open follow-on (real gossip-fed convergence still active) |",
    "#3228 -> #3229 -> #3313",
    "#3228 -> #3413 -> #3414",
    "#3333 -> #3424 -> #3425 -> #3426",
    "Historical Baseline (Superseded)",
)
for marker in required_markers:
    if marker not in text:
        raise SystemExit(f"marker_missing:{marker}")

for reason, snippet in (
    ("ingress_hand_rolled_tcp_listener", "Hand-rolled `TcpListener` + manual HTTP parser"),
    ("storage_file_only_no_sqlite", "File-based JSON dumps"),
    ("runtime_mode_full_missing", "A production node needs to be a daemon AND serve an API"),
):
    if snippet in text:
        raise SystemExit(f"stale_claim_detected:{reason}")
PY

cp "$PLAN_DOC" "$TMP_TAMPERED"
cat >>"$TMP_TAMPERED" <<'TXT'
Stale baseline regression fixture:
Hand-rolled `TcpListener` + manual HTTP parser
TXT

set +e
tampered_output="$(
  python3 - "$TMP_TAMPERED" 2>&1 <<'PY'
from __future__ import annotations

import pathlib
import sys

doc_path = pathlib.Path(sys.argv[1])
text = doc_path.read_text(encoding="utf-8")

for reason, snippet in (
    ("ingress_hand_rolled_tcp_listener", "Hand-rolled `TcpListener` + manual HTTP parser"),
    ("storage_file_only_no_sqlite", "File-based JSON dumps"),
    ("runtime_mode_full_missing", "A production node needs to be a daemon AND serve an API"),
):
    if snippet in text:
        raise SystemExit(f"stale_claim_detected:{reason}")
PY
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered production-service next-steps doc to fail stale-claim contract" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'stale_claim_detected:ingress_hand_rolled_tcp_listener'; then
  echo "expected deterministic stale-claim marker for tampered production-service next-steps doc" >&2
  exit 1
fi

echo "production-service next-steps docs contract tests passed."
