use std::path::Path;

pub(super) fn ensure_binary_path_is_executable(path: &str, label: &str) -> Result<(), String> {
    let binary_path = Path::new(path);
    if !binary_path.is_absolute() {
        return Err(format!(
            "external execution preflight failed: {label} binary path must be absolute: {path}"
        ));
    }
    if !binary_path.exists() {
        return Err(format!(
            "external execution preflight failed: {label} binary not found: {path}"
        ));
    }
    if !binary_path.is_file() {
        return Err(format!(
            "external execution preflight failed: {label} binary path is not a file: {path}"
        ));
    }
    ensure_binary_executable(binary_path, label)
}

#[cfg(unix)]
pub(super) fn ensure_binary_executable(path: &Path, label: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mode = std::fs::metadata(path)
        .map_err(|err| {
            format!(
                "external execution preflight failed: {label} binary metadata read failed: {} ({err})",
                path.display()
            )
        })?
        .permissions()
        .mode();
    if mode & 0o111 == 0 {
        return Err(format!(
            "external execution preflight failed: {label} binary is not executable: {}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
pub(super) fn ensure_binary_executable(_path: &Path, _label: &str) -> Result<(), String> {
    Ok(())
}
