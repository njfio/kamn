use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use kamn_e2e_harness::LiveTaskEvidencePaths;

pub(crate) fn write(root: &Path, actors: &[String; 3], live: &LiveTaskEvidencePaths, state: &Path) {
    let script = format!(
        r#"#!/bin/sh
case " $* " in *" --print "*) echo KAMN_PI_PREFLIGHT_OK; exit 0;; esac
role=""; previous=""
for arg in "$@"; do
  if [ "$previous" = "--name" ]; then role="$arg"; fi
  previous="$arg"
done
test -n "$KAMN_MVP_LIVE_MCP_BINARY" || exit 41
test -n "$KAMN_MVP_LIVE_MCP_ENDPOINT" || exit 42
test -n "$KAMN_MVP_LIVE_MCP_AGENT_A_NAME" || exit 43
test -n "$KAMN_MVP_LIVE_MCP_AGENT_B_NAME" || exit 44
test -n "$KAMN_MVP_LIVE_MCP_AGENT_C_NAME" || exit 45
while read line; do
  case "$line" in *'"type":"abort"'*) continue;; esac
  mkdir -p "$(dirname "$KAMN_MVP_PI_TRANSACTION_AGENT_A_FILE")"
  cp "{}" "$KAMN_MVP_PI_TRANSACTION_AGENT_A_FILE"
  cp "{}" "$KAMN_MVP_PI_TRANSACTION_AGENT_B_FILE"
  cp "{}" "$KAMN_MVP_PI_TRANSACTION_AGENT_C_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_HANDOFF_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_AGENT_C_OBSERVATION_FILE"
  cp "{}" "{}"
  echo '{{"type":"response","command":"prompt","success":true}}'
  if [ "$role" = "kamn-mvp-agent-b" ]; then
    echo '{{"type":"agent_end","messages":[{{"did":"kamn:did:agent:b"}}]}}'
  else
    echo '{{"type":"agent_end","messages":[]}}'
  fi
done
"#,
        actors[0],
        actors[1],
        actors[2],
        live.handoff,
        live.agent_a_receipt,
        live.agent_b_receipt,
        live.agent_c_observation,
        state.display(),
        root.join("staging/service-api-state.json").display(),
    );
    write_executable(root.join("pi").as_path(), script.as_str());
}

fn write_executable(path: &Path, body: &str) {
    std::fs::write(path, body).expect("fake Pi");
    let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
}
