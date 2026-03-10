use super::*;
pub(crate) fn generate_test_ca_signed_certificate_chain(temp_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let ca_cert_path = temp_dir.join("ca-cert.pem");
    let ca_key_path = temp_dir.join("ca-key.pem");
    let server_key_path = temp_dir.join("server-key.pem");
    let server_csr_path = temp_dir.join("server.csr");
    let server_cert_path = temp_dir.join("server-cert.pem");
    let server_extensions_path = temp_dir.join("server-ext.cnf");

    let ca_status = Command::new("openssl")
        .arg("req")
        .arg("-x509")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-keyout")
        .arg(ca_key_path.as_os_str())
        .arg("-out")
        .arg(ca_cert_path.as_os_str())
        .arg("-days")
        .arg("1")
        .arg("-nodes")
        .arg("-subj")
        .arg("/CN=kamn-test-ca")
        .arg("-addext")
        .arg("basicConstraints = critical,CA:TRUE")
        .arg("-addext")
        .arg("keyUsage = critical,keyCertSign,cRLSign")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for CA certificate generation");
    assert!(
        ca_status.success(),
        "openssl CA certificate generation should succeed"
    );

    let csr_status = Command::new("openssl")
        .arg("req")
        .arg("-new")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-keyout")
        .arg(server_key_path.as_os_str())
        .arg("-out")
        .arg(server_csr_path.as_os_str())
        .arg("-nodes")
        .arg("-subj")
        .arg("/CN=127.0.0.1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for server csr generation");
    assert!(
        csr_status.success(),
        "openssl server csr generation should succeed"
    );

    fs::write(
        server_extensions_path.as_path(),
        "subjectAltName = DNS:localhost,IP:127.0.0.1\nbasicConstraints = critical,CA:FALSE\nkeyUsage = critical,digitalSignature,keyEncipherment\nextendedKeyUsage = serverAuth\n",
    )
    .expect("server extension file should be written");

    let sign_status = Command::new("openssl")
        .arg("x509")
        .arg("-req")
        .arg("-in")
        .arg(server_csr_path.as_os_str())
        .arg("-CA")
        .arg(ca_cert_path.as_os_str())
        .arg("-CAkey")
        .arg(ca_key_path.as_os_str())
        .arg("-CAcreateserial")
        .arg("-out")
        .arg(server_cert_path.as_os_str())
        .arg("-days")
        .arg("1")
        .arg("-sha256")
        .arg("-extfile")
        .arg(server_extensions_path.as_os_str())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for server certificate signing");
    assert!(
        sign_status.success(),
        "openssl server certificate signing should succeed"
    );

    (ca_cert_path, server_cert_path, server_key_path)
}

pub(crate) fn spawn_https_single_request_server(
    status_code: u16,
    response_body: &str,
) -> HttpsSingleRequestServer {
    let temp_dir = unique_temp_dir("kolme-https-server");
    let (ca_cert_path, server_cert_path, server_key_path) =
        generate_test_ca_signed_certificate_chain(temp_dir.as_path());
    let server_script = r#"
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

    let mut child = Command::new("python3")
        .arg("-u")
        .arg("-c")
        .arg(server_script)
        .arg("0")
        .arg(server_cert_path.as_os_str())
        .arg(server_key_path.as_os_str())
        .arg(status_code.to_string())
        .arg(response_body)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("python https test server should spawn");

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

    let port = port_line
        .trim()
        .parse::<u16>()
        .expect("python https test server should emit a valid port");
    HttpsSingleRequestServer {
        base_url: format!("https://127.0.0.1:{port}"),
        ca_cert_path,
        child,
        temp_dir,
    }
}

