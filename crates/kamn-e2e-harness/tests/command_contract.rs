mod support;
#[path = "command_contract/shared_support.rs"]
mod shared_support;

use kamn_e2e_harness::{
    all_orchestration_phases, all_phase_result_statuses, execute_run_contract,
    execute_verify_contract, parse_command_args, parse_scenario_csv, HarnessCommand,
    RunCommandConfig, VerifyCommandConfig,
};
use support::command_contract_support::{
    set_executable, set_non_executable, temp_path, valid_chain_dump_json,
    with_external_component_binaries, write_stub_binary,
};

#[path = "command_contract/parser_verify_contract_tests.rs"]
mod parser_verify_contract_tests;
#[path = "command_contract/phase_inventory_contract_tests.rs"]
mod phase_inventory_contract_tests;
#[path = "command_contract/integration_runtime_contract_tests.rs"]
mod integration_runtime_contract_tests;
#[path = "command_contract/external_execution_contract_tests.rs"]
mod external_execution_contract_tests;
#[path = "command_contract/scenario_evidence_contract_tests.rs"]
mod scenario_evidence_contract_tests;
#[path = "command_contract/teardown_contract_tests.rs"]
mod teardown_contract_tests;
