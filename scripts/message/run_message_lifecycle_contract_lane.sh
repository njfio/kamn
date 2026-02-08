#!/usr/bin/env bash
set -euo pipefail

cargo test -p kamn-core --lib message_lifecycle::tests:: >/dev/null
cargo test -p kamn-core --test message_lifecycle_docs >/dev/null

echo "message lifecycle snapshot contract lane tests passed."
