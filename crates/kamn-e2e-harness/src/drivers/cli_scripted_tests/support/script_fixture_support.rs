use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
}

pub(crate) fn script_path_str(script_path: &Path) -> &str {
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
