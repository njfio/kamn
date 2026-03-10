use super::*;

pub(crate) fn generate_test_ca_signed_certificate_chain(
    temp_dir: &Path,
) -> (PathBuf, PathBuf, PathBuf) {
    let ca_cert_path = temp_dir.join("ca-cert.pem");
    let ca_key_path = temp_dir.join("ca-key.pem");
    let server_key_path = temp_dir.join("server-key.pem");
    let server_csr_path = temp_dir.join("server.csr");
    let server_cert_path = temp_dir.join("server-cert.pem");
    let server_extensions_path = temp_dir.join("server-ext.cnf");
    generate_ca_certificate(ca_cert_path.as_path(), ca_key_path.as_path());
    generate_server_csr(server_key_path.as_path(), server_csr_path.as_path());
    write_server_extensions(server_extensions_path.as_path());
    sign_server_certificate(
        ca_cert_path.as_path(),
        ca_key_path.as_path(),
        server_csr_path.as_path(),
        server_cert_path.as_path(),
        server_extensions_path.as_path(),
    );
    (ca_cert_path, server_cert_path, server_key_path)
}

fn generate_ca_certificate(ca_cert_path: &Path, ca_key_path: &Path) {
    let status = Command::new("openssl")
        .args(["req", "-x509", "-newkey", "rsa:2048"])
        .arg("-keyout")
        .arg(ca_key_path.as_os_str())
        .arg("-out")
        .arg(ca_cert_path.as_os_str())
        .args(["-days", "1", "-nodes", "-subj", "/CN=kamn-test-ca"])
        .args(["-addext", "basicConstraints = critical,CA:TRUE"])
        .args(["-addext", "keyUsage = critical,keyCertSign,cRLSign"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for CA certificate generation");
    assert!(
        status.success(),
        "openssl CA certificate generation should succeed"
    );
}

fn generate_server_csr(server_key_path: &Path, server_csr_path: &Path) {
    let status = Command::new("openssl")
        .args(["req", "-new", "-newkey", "rsa:2048"])
        .arg("-keyout")
        .arg(server_key_path.as_os_str())
        .arg("-out")
        .arg(server_csr_path.as_os_str())
        .args(["-nodes", "-subj", "/CN=127.0.0.1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for server csr generation");
    assert!(
        status.success(),
        "openssl server csr generation should succeed"
    );
}

fn write_server_extensions(server_extensions_path: &Path) {
    fs::write(
        server_extensions_path,
        "subjectAltName = DNS:localhost,IP:127.0.0.1\nbasicConstraints = critical,CA:FALSE\nkeyUsage = critical,digitalSignature,keyEncipherment\nextendedKeyUsage = serverAuth\n",
    )
    .expect("server extension file should be written");
}

fn sign_server_certificate(
    ca_cert_path: &Path,
    ca_key_path: &Path,
    server_csr_path: &Path,
    server_cert_path: &Path,
    server_extensions_path: &Path,
) {
    let status = sign_server_certificate_command(
        ca_cert_path,
        ca_key_path,
        server_csr_path,
        server_cert_path,
        server_extensions_path,
    )
    .status()
    .expect("openssl should run for server certificate signing");
    assert!(
        status.success(),
        "openssl server certificate signing should succeed"
    );
}

fn sign_server_certificate_command(
    ca_cert_path: &Path,
    ca_key_path: &Path,
    server_csr_path: &Path,
    server_cert_path: &Path,
    server_extensions_path: &Path,
) -> Command {
    let mut command = Command::new("openssl");
    command
        .args(["x509", "-req", "-in"])
        .arg(server_csr_path.as_os_str())
        .arg("-CA")
        .arg(ca_cert_path.as_os_str())
        .arg("-CAkey")
        .arg(ca_key_path.as_os_str())
        .args(["-CAcreateserial", "-out"])
        .arg(server_cert_path.as_os_str())
        .args(["-days", "1", "-sha256", "-extfile"])
        .arg(server_extensions_path.as_os_str())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}
