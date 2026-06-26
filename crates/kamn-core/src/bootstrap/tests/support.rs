use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const CONTENT_STORE_FIXTURE: &str = "content-store.snapshot";
pub const TASK_OPERATION_STORE_FIXTURE: &str = "task-operation.snapshot";
pub const CHANNEL_STORE_FIXTURE: &str = "channel.snapshot";
pub const MESSAGE_LIFECYCLE_STORE_FIXTURE: &str = "message-lifecycle.snapshot";
pub const RUNTIME_SNAPSHOT_STORE_FIXTURE: &str = "runtime.snapshot";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn temp_storage_dir(tag: &str) -> PathBuf {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-bootstrap-{tag}-{}-{counter}-{nonce}",
        std::process::id(),
    ))
}

pub fn write_fixture(path: PathBuf, contents: &str) {
    fs::create_dir_all(path.parent().expect("fixture path must have parent"))
        .expect("fixture directory should build");
    fs::write(path, contents).expect("fixture should write");
}
