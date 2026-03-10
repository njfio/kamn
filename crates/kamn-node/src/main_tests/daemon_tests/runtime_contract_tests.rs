// daemon runtime contract decomposition: route transition, shutdown, parse, output,
// phase6, and selector-bundle assertions through bounded include modules.
include!("runtime_contract_tests/support.rs");
include!("runtime_contract_tests/structured_transition_contract_tests.rs");
include!("runtime_contract_tests/structured_shutdown_contract_tests.rs");
include!("runtime_contract_tests/parse_control_contract_tests.rs");
include!("runtime_contract_tests/completion_output_contract_tests.rs");
include!("runtime_contract_tests/phase6_projection_contract_tests.rs");
include!("runtime_contract_tests/selector_bundle_contract_tests.rs");
