use super::*;

pub(crate) struct HttpsSingleRequestServer {
    pub(crate) base_url: String,
    pub(crate) ca_cert_path: PathBuf,
    child: Child,
    pub(crate) temp_dir: PathBuf,
}

impl HttpsSingleRequestServer {
    pub(crate) fn wait_for_exit(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => assert_wait_deadline(&mut self.child, deadline),
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

pub(crate) fn spawn_https_single_request_server(
    status_code: u16,
    response_body: &str,
) -> HttpsSingleRequestServer {
    let temp_dir = unique_temp_dir("sdk-https-server");
    let (ca_cert_path, server_cert_path, server_key_path) =
        certificate_chain_support::generate_test_ca_signed_certificate_chain(temp_dir.as_path());
    let mut child = spawn_https_server(
        server_cert_path.as_path(),
        server_key_path.as_path(),
        status_code,
        response_body,
    );
    let port = read_bound_port(&mut child);
    HttpsSingleRequestServer {
        base_url: format!("https://127.0.0.1:{port}"),
        ca_cert_path,
        child,
        temp_dir,
    }
}

fn assert_wait_deadline(child: &mut Child, deadline: Instant) {
    if Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
        return;
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("https test server did not exit after handling request");
}

fn spawn_https_server(
    server_cert_path: &Path,
    server_key_path: &Path,
    status_code: u16,
    response_body: &str,
) -> Child {
    Command::new("python3")
        .arg("-u")
        .arg("-c")
        .arg(https_server_script())
        .arg("0")
        .arg(server_cert_path.as_os_str())
        .arg(server_key_path.as_os_str())
        .arg(status_code.to_string())
        .arg(response_body)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("python https test server should spawn")
}

fn read_bound_port(child: &mut Child) -> u16 {
    let stdout = child
        .stdout
        .take()
        .expect("python https test server stdout should be piped");
    let mut stdout_reader = BufReader::new(stdout);
    let mut port_line = String::new();
    stdout_reader
        .read_line(&mut port_line)
        .expect("python https test server should emit bound port");
    child.stdout = Some(stdout_reader.into_inner());
    port_line
        .trim()
        .parse::<u16>()
        .expect("python https test server should emit a valid port")
}

const HTTPS_SERVER_SCRIPT: &str = r#"
import http.server
import ssl
import sys

port = int(sys.argv[1])
cert_file = sys.argv[2]
key_file = sys.argv[3]
status_code = int(sys.argv[4])
response_body = sys.argv[5].encode("utf-8")

class Handler(http.server.BaseHTTPRequestHandler):
    def _reply(self):
        if "Content-Length" in self.headers:
            _ = self.rfile.read(int(self.headers["Content-Length"]))
        self.send_response(status_code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(response_body)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(response_body)

    def do_POST(self):
        self._reply()

    def do_GET(self):
        self._reply()

    def log_message(self, _format, *args):
        return

httpd = http.server.HTTPServer(("127.0.0.1", port), Handler)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(certfile=cert_file, keyfile=key_file)
httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
print(httpd.server_address[1], flush=True)
httpd.handle_request()
"#;

fn https_server_script() -> &'static str {
    HTTPS_SERVER_SCRIPT
}
