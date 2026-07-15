use super::build_command;

#[test]
fn default_proof_command_targets_only_the_kamn_node_binary() {
    let command = build_command(None, "proof-test").expect("default command");
    let args = command
        .get_args()
        .map(|value| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        args,
        [
            "test",
            "-p",
            "kamn-node",
            "--bin",
            "kamn-node",
            "proof-test",
            "--",
            "--nocapture",
        ]
    );
}
