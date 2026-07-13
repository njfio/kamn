use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(super) fn install_fake_solana(root: &Path) -> String {
    std::fs::write(root.join("payer.json"), "[]").expect("payer fixture");
    write_executable(
        &root.join("solana-keygen"),
        "#!/bin/sh\necho 2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T\n",
    );
    write_executable(&root.join("solana"), &solana_script(root));
    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{original}", root.display()));
    original
}

fn solana_script(root: &Path) -> String {
    format!(
        r#"#!/bin/sh
echo "$*" >> "{}/solana-calls.log"
cat <<'JSON'
{{"confirmationStatus":"finalized","meta":{{"err":null,"preBalances":[2500000000,2500000000],"postBalances":[2498995000,2501000000]}},"transaction":{{"signatures":["devnet-signature-111"],"message":{{"accountKeys":["2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T","FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe"]}}}}}}
JSON
"#,
        root.display()
    )
}

fn write_executable(path: &Path, body: &str) {
    std::fs::write(path, body).expect("fake executable");
    let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
}
