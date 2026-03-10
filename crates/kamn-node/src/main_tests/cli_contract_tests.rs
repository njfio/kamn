use super::*;

#[path = "cli_contract_tests/support.rs"]
mod support;

#[path = "cli_contract_tests/required_argument_contract_tests.rs"]
mod required_argument_contract_tests;
#[path = "cli_contract_tests/kolme_live_contract_tests.rs"]
mod kolme_live_contract_tests;
#[path = "cli_contract_tests/parse_validation_contract_tests.rs"]
mod parse_validation_contract_tests;
#[path = "cli_contract_tests/runtime_regression_contract_tests.rs"]
mod runtime_regression_contract_tests;

pub(crate) use support::*;
