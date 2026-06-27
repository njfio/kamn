use super::*;

pub(crate) fn ensure_test_service_auth_private_key() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        env::set_var(
            SERVICE_AUTH_PRIVATE_KEY_ENV,
            TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
        )
    });
}

pub(crate) fn reserve_loopback_addr() -> String {
    let listener = bind_loopback_listener();
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(crate) fn bind_loopback_listener() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener nonblocking mode should configure");
    listener
}

pub(crate) fn tls_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    pub(crate) fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        set_env_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        set_env_var(self.key, self.previous.as_deref());
    }
}

fn set_env_var(key: &'static str, value: Option<&str>) {
    match value {
        Some(value) => env::set_var(key, value),
        None => env::remove_var(key),
    }
}

pub(crate) fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}
