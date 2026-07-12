use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn configure(root: &Path, env: &mut BTreeMap<String, String>) {
    write_runtime(root);
    std::fs::write(root.join("kamn-mcp-server"), "stub").expect("MCP binary");
    env.insert(
        "KAMN_MVP_LOCAL_NODE_BINARY".to_owned(),
        root.join("kamn-node").display().to_string(),
    );
    env.insert(
        "KAMN_MVP_LIVE_MCP_BINARY".to_owned(),
        root.join("kamn-mcp-server").display().to_string(),
    );
    env.insert(
        "KAMN_MVP_LIVE_MCP_ENDPOINT".to_owned(),
        format!("http://127.0.0.1:{}", free_port()),
    );
}

fn write_runtime(root: &Path) {
    let script = format!(
        r#"#!/bin/sh
port=""
previous=""
for arg in "$@"; do
  if [ "$previous" = "--api-bind" ]; then port="${{arg##*:}}"; fi
  previous="$arg"
done
exec python3 -c 'import signal,socket,sys,time
root=sys.argv[1]; port=int(sys.argv[2])
open(root+"/runtime.started","w").write(str(__import__("os").getpid()))
server=socket.socket(); server.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1); server.bind(("127.0.0.1",port)); server.listen()
def stop(*_):
 open(root+"/runtime.stopped","w").write("stopped"); server.close(); sys.exit(0)
signal.signal(signal.SIGTERM,stop)
while True: time.sleep(.05)' "{}" "$port"
"#,
        root.display()
    );
    let path = root.join("kamn-node");
    std::fs::write(&path, script).expect("fake runtime");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("free port")
        .local_addr()
        .expect("local address")
        .port()
}
