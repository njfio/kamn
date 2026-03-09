use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TOOL_RESPONSE_TEMPLATE: &str = r#"#!/usr/bin/env python3
import sys
init_payload = __INIT_PAYLOAD__
tool_payload = __TOOL_PAYLOAD__
sys.stdout.write(
    f"Content-Length: {len(init_payload)}\r\n\r\n{init_payload}"
    f"Content-Length: {len(tool_payload)}\r\n\r\n{tool_payload}"
)
"#;

pub(crate) fn unique_temp_script_path(stem: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{stem}-{}-{nonce}.py", std::process::id()))
}

pub(crate) fn write_executable_python_script(script_path: &Path, source: &str) {
    fs::write(script_path, source).expect("script fixture should be written");
    let mut permissions = executable_permissions(script_path);
    permissions.set_mode(0o755);
    fs::set_permissions(script_path, permissions).expect("script fixture should be executable");
    std::thread::sleep(Duration::from_millis(5));
}

pub(crate) fn write_mcp_tool_response_script(
    script_path: &Path,
    request_id: &str,
    result_payload: &str,
) {
    let init_payload =
        r#"{"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}"#;
    let tool_payload =
        format!(r#"{{"jsonrpc":"2.0","id":"{request_id}","result":{result_payload}}}"#);
    let script_source = render_tool_response_script(init_payload, tool_payload.as_str());
    write_executable_python_script(script_path, script_source.as_str());
}

pub(crate) fn script_path_str<'a>(script_path: &'a Path) -> &'a str {
    script_path
        .to_str()
        .expect("script path should be valid utf-8")
}

pub(crate) fn remove_script(script_path: &Path) {
    fs::remove_file(script_path).expect("script fixture should be removable");
}

fn executable_permissions(script_path: &Path) -> fs::Permissions {
    fs::metadata(script_path)
        .expect("script metadata should be readable")
        .permissions()
}

fn render_tool_response_script(init_payload: &str, tool_payload: &str) -> String {
    TOOL_RESPONSE_TEMPLATE
        .replace("__INIT_PAYLOAD__", &format!("{init_payload:?}"))
        .replace("__TOOL_PAYLOAD__", &format!("{tool_payload:?}"))
}
