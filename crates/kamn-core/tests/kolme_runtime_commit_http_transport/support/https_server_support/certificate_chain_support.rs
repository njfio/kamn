use super::*;

pub(crate) fn generate_test_ca_signed_certificate_chain(
    temp_dir: &Path,
) -> (PathBuf, PathBuf, PathBuf) {
    let paths = certificate_paths(temp_dir);
    run_openssl_ca_request(paths.ca_cert_path.as_path(), paths.ca_key_path.as_path());
    run_openssl_server_csr(
        paths.server_key_path.as_path(),
        paths.server_csr_path.as_path(),
    );
    write_server_extensions(paths.server_extensions_path.as_path());
    run_openssl_server_sign(paths.as_refs());
    (
        paths.ca_cert_path,
        paths.server_cert_path,
        paths.server_key_path,
    )
}

struct CertificatePaths {
    ca_cert_path: PathBuf,
    ca_key_path: PathBuf,
    server_key_path: PathBuf,
    server_csr_path: PathBuf,
    server_cert_path: PathBuf,
    server_extensions_path: PathBuf,
}

impl CertificatePaths {
    fn as_refs(&self) -> CertificatePathRefs<'_> {
        CertificatePathRefs {
            ca_cert_path: self.ca_cert_path.as_path(),
            ca_key_path: self.ca_key_path.as_path(),
            server_csr_path: self.server_csr_path.as_path(),
            server_cert_path: self.server_cert_path.as_path(),
            server_extensions_path: self.server_extensions_path.as_path(),
        }
    }
}

struct CertificatePathRefs<'a> {
    ca_cert_path: &'a Path,
    ca_key_path: &'a Path,
    server_csr_path: &'a Path,
    server_cert_path: &'a Path,
    server_extensions_path: &'a Path,
}

fn certificate_paths(temp_dir: &Path) -> CertificatePaths {
    CertificatePaths {
        ca_cert_path: temp_dir.join("ca-cert.pem"),
        ca_key_path: temp_dir.join("ca-key.pem"),
        server_key_path: temp_dir.join("server-key.pem"),
        server_csr_path: temp_dir.join("server.csr"),
        server_cert_path: temp_dir.join("server-cert.pem"),
        server_extensions_path: temp_dir.join("server-ext.cnf"),
    }
}

fn run_openssl_ca_request(ca_cert_path: &Path, ca_key_path: &Path) {
    let status = Command::new("openssl")
        .args([
            "req", "-x509", "-newkey", "rsa:2048", "-days", "1", "-nodes",
        ])
        .arg("-keyout")
        .arg(ca_key_path)
        .arg("-out")
        .arg(ca_cert_path)
        .args([
            "-subj",
            "/CN=kamn-test-ca",
            "-addext",
            "basicConstraints = critical,CA:TRUE",
            "-addext",
            "keyUsage = critical,keyCertSign,cRLSign",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for CA certificate generation");
    assert!(
        status.success(),
        "openssl CA certificate generation should succeed"
    );
}

fn run_openssl_server_csr(server_key_path: &Path, server_csr_path: &Path) {
    let status = Command::new("openssl")
        .args(["req", "-new", "-newkey", "rsa:2048", "-nodes"])
        .arg("-keyout")
        .arg(server_key_path)
        .arg("-out")
        .arg(server_csr_path)
        .args(["-subj", "/CN=127.0.0.1"])
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

fn run_openssl_server_sign(paths: CertificatePathRefs<'_>) {
    let status = Command::new("openssl")
        .args(["x509", "-req", "-days", "1", "-sha256", "-CAcreateserial"])
        .arg("-in")
        .arg(paths.server_csr_path)
        .arg("-CA")
        .arg(paths.ca_cert_path)
        .arg("-CAkey")
        .arg(paths.ca_key_path)
        .arg("-out")
        .arg(paths.server_cert_path)
        .arg("-extfile")
        .arg(paths.server_extensions_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for server certificate signing");
    assert!(
        status.success(),
        "openssl server certificate signing should succeed"
    );
}
