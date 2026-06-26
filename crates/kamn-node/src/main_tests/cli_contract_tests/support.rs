use super::*;
use crate::NodeCli;

pub(crate) fn cli_args() -> Vec<String> {
    vec!["kamn-node".to_owned()]
}

pub(crate) fn push_flag(args: &mut Vec<String>, flag: &str) {
    args.push(flag.to_owned());
}

pub(crate) fn push_pair(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_owned());
    args.push(value.to_owned());
}

pub(crate) fn push_pairs(args: &mut Vec<String>, pairs: &[(&str, &str)]) {
    for (flag, value) in pairs {
        push_pair(args, flag, value);
    }
}

pub(crate) fn with_pairs(mut args: Vec<String>, pairs: &[(&str, &str)]) -> Vec<String> {
    push_pairs(&mut args, pairs);
    args
}

pub(crate) fn approver_args() -> Vec<String> {
    with_pairs(cli_args(), &[("--role", "approver")])
}

pub(crate) fn processor_runtime_args(mode: &str) -> Vec<String> {
    with_pairs(
        cli_args(),
        &[("--role", "processor"), ("--runtime-mode", mode)],
    )
}

pub(crate) fn planning_args() -> Vec<String> {
    processor_runtime_args("planning")
}

pub(crate) fn recovery_args() -> Vec<String> {
    processor_runtime_args("recovery-check")
}

pub(crate) fn daemon_args() -> Vec<String> {
    processor_runtime_args("daemon")
}

pub(crate) fn full_args() -> Vec<String> {
    processor_runtime_args("full")
}

pub(crate) fn kolme_live_args() -> Vec<String> {
    processor_runtime_args("kolme-live")
}

pub(crate) fn kolme_live_declared_args() -> Vec<String> {
    with_pairs(
        kolme_live_args(),
        &[
            ("--kolme-live-base-url", "http://127.0.0.1:3000"),
            ("--kolme-live-provider-hint", "kolme-fork-local"),
            ("--kolme-live-signing-profile", "kolme-fork-secp256k1-v1"),
        ],
    )
}

pub(crate) fn strict_kolme_live_args() -> Vec<String> {
    let mut args = kolme_live_declared_args();
    push_flag(&mut args, "--kolme-live-strict-signer-contracts");
    args
}

pub(crate) fn strict_kolme_live_env_local_args() -> Vec<String> {
    with_pairs(
        strict_kolme_live_args(),
        &[
            ("--kolme-live-signer-profile", "ops-primary"),
            ("--kolme-live-signer-key-source", "env-local"),
        ],
    )
}

pub(crate) fn managed_external_strict_kolme_live_args() -> Vec<String> {
    with_pairs(
        strict_kolme_live_args(),
        &[
            ("--kolme-live-signer-profile", "ops-primary"),
            ("--kolme-live-signer-key-source", "managed-external"),
        ],
    )
}

pub(crate) fn assert_parse_error(args: Vec<String>, expected: ConfigError) {
    assert_eq!(parse_args(args), Err(expected));
}

pub(crate) fn missing_arg(flag: &'static str) -> ConfigError {
    ConfigError::MissingArgumentValue(flag)
}

pub(crate) fn parse_cli(args: Vec<String>, context: &str) -> NodeCli {
    parse_args(args).unwrap_or_else(|error| panic!("{context}: {error:?}"))
}

pub(crate) fn execute_cli(args: Vec<String>, context: &str) -> NodeBootstrapReport {
    let parsed = parse_cli(args, context);
    execute(parsed).unwrap_or_else(|error| panic!("{context}: {error:?}"))
}
