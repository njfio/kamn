use super::*;
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
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            env::set_var(self.key, previous);
        } else {
            env::remove_var(self.key);
        }
    }
}

pub(crate) struct HttpsSingleRequestServer {
    pub(crate) base_url: String,
    pub(crate) ca_cert_path: PathBuf,
    pub(crate) child: Child,
    pub(crate) temp_dir: PathBuf,
}

impl HttpsSingleRequestServer {
    pub(crate) fn wait_for_exit(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        panic!("https test server did not exit after handling request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("failed to wait for https test server exit: {error}"),
            }
        }
    }
}

impl Drop for HttpsSingleRequestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}
