use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn write(root: &Path, mode: &str) {
    let b_branch = match mode {
        "hang" => "if [ \"$role\" = \"kamn-mvp-agent-b\" ]; then read line; sleep 2; exit 9; fi",
        "fail" => "if [ \"$role\" = \"kamn-mvp-agent-b\" ]; then read line; exit 9; fi",
        "tool-error" => "if [ \"$role\" = \"kamn-mvp-agent-b\" ]; then read line; echo '{\"type\":\"tool_execution_end\",\"toolName\":\"kamn_live_agent_b_register\",\"result\":{},\"isError\":true}'; echo '{\"type\":\"agent_end\",\"messages\":[]}'; fi",
        _ => "",
    };
    let script = format!(
        r#"#!/bin/sh
case " $* " in *" --print "*) echo KAMN_PI_PREFLIGHT_OK; exit 0;; esac
role=""
previous=""
for arg in "$@"; do
  if [ "$previous" = "--name" ]; then role="$arg"; fi
  previous="$arg"
done
echo $$ > "{}/$role.pid"
trap 'echo cleaned > "{}/$role.cleaned"; exit 0' TERM INT
{b_branch}
while read line; do
  echo "$role" >> "{}/prompts.log"
  echo '{{"type":"response","command":"prompt","success":true}}'
  if [ "$role" = "kamn-mvp-agent-b" ]; then
    echo '{{"type":"tool_execution_end","toolName":"kamn_live_agent_b_register","result":{{"details":{{"result":{{"did":"kamn:did:agent:b"}}}}}},"isError":false}}'
    echo '{{"type":"agent_end","messages":[]}}'
  else
    echo '{{"type":"agent_end","messages":[]}}'
  fi
done
echo cleaned > "{}/$role.cleaned"
"#,
        root.display(),
        root.display(),
        root.display(),
        root.display()
    );
    let path = root.join("pi");
    std::fs::write(&path, script).expect("fake Pi");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
}
